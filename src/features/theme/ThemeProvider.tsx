import {
	createContext,
	type ReactNode,
	useCallback,
	useContext,
	useEffect,
	useMemo,
	useState,
} from "react";
import {
	commands,
	type ColorMode,
	type SettingsSnapshotDto,
	type ThemeId,
	type ThemeKind,
} from "../../bindings";

const DEFAULT_THEME_KIND: ThemeKind = "heroui";
const DEFAULT_THEME_ID: ThemeId = "default";
const DEFAULT_ACCENT_COLOR = null;
const DEFAULT_COLOR_MODE: ColorMode = "system";

const CATPPUCCIN_THEMES = new Set<ThemeId>([
	"latte",
	"frappe",
	"macchiato",
	"mocha",
]);

export interface ThemeConfig {
	themeKind: ThemeKind;
	themeId: ThemeId;
	accentColor: string | null;
	colorMode: ColorMode;
}

const DEFAULT_THEME_CONFIG: ThemeConfig = {
	themeKind: DEFAULT_THEME_KIND,
	themeId: DEFAULT_THEME_ID,
	accentColor: DEFAULT_ACCENT_COLOR,
	colorMode: DEFAULT_COLOR_MODE,
};

interface ThemeContextType {
	themeKind: ThemeKind;
	themeId: ThemeId;
	accentColor: string | null;
	colorMode: ColorMode;
	effectiveMode: "light" | "dark";
	setThemeConfig: (config: ThemeConfig) => Promise<void>;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: ReactNode }) {
	const [themeConfig, setThemeConfigState] =
		useState<ThemeConfig>(DEFAULT_THEME_CONFIG);
	const [systemPrefersDark, setSystemPrefersDark] = useState(() =>
		window.matchMedia("(prefers-color-scheme: dark)").matches,
	);

	const resolveHeroMode = useCallback(
		(nextConfig: ThemeConfig): "light" | "dark" => {
			if (nextConfig.colorMode === "light") return "light";
			if (nextConfig.colorMode === "dark") return "dark";
			return systemPrefersDark ? "dark" : "light";
		},
		[systemPrefersDark],
	);

	const effectiveMode = useMemo(() => {
		if (themeConfig.themeKind === "catppuccin") {
			return themeConfig.themeId === "latte" ? "light" : "dark";
		}
		return resolveHeroMode(themeConfig);
	}, [themeConfig, resolveHeroMode]);

	const applyThemeToDocument = useCallback((nextConfig: ThemeConfig) => {
		const root = document.documentElement;
		if (nextConfig.themeKind === "catppuccin") {
			const catppuccinTheme = CATPPUCCIN_THEMES.has(nextConfig.themeId)
				? nextConfig.themeId
				: "mocha";
			const isLightTheme = catppuccinTheme === "latte";
			root.classList.toggle("light", isLightTheme);
			root.classList.toggle("dark", !isLightTheme);
			root.setAttribute("data-theme", catppuccinTheme);
			root.style.colorScheme = isLightTheme ? "light" : "dark";
			root.style.removeProperty("--accent");
			root.style.removeProperty("--accent-foreground");
		} else {
			const mode = resolveHeroMode(nextConfig);
			root.classList.toggle("dark", mode === "dark");
			root.classList.toggle("light", mode === "light");
			root.setAttribute("data-theme", mode);
			root.style.colorScheme = mode;
			if (nextConfig.accentColor) {
				root.style.setProperty("--accent", nextConfig.accentColor);
				root.style.setProperty("--accent-foreground", "var(--snow)");
			} else {
				root.style.removeProperty("--accent");
				root.style.removeProperty("--accent-foreground");
			}
		}
		localStorage.setItem("theme-config", JSON.stringify(nextConfig));
	}, [resolveHeroMode]);

	useEffect(() => {
		applyThemeToDocument(themeConfig);
	}, [themeConfig, applyThemeToDocument]);

	useEffect(() => {
		const media = window.matchMedia("(prefers-color-scheme: dark)");
		const onChange = (event: MediaQueryListEvent) => {
			setSystemPrefersDark(event.matches);
		};
		media.addEventListener("change", onChange);
		return () => {
			media.removeEventListener("change", onChange);
		};
	}, []);

	useEffect(() => {
		let cancelled = false;
		const hydrateTheme = async () => {
			try {
				const result = await commands.getSettings();
				if (!cancelled && result.status === "ok") {
					setThemeConfigState(normalizeThemeSettings(result.data));
					return;
				}
			} catch {
				// Fall back to locally cached value when IPC is unavailable.
			}
			if (!cancelled) {
				const raw = localStorage.getItem("theme-config");
				if (!raw) return;
				try {
					const parsed = JSON.parse(raw) as ThemeConfig;
					setThemeConfigState(normalizeThemeSettings(parsed));
				} catch {
					// ignore stale local cache
				}
			}
		};
		void hydrateTheme();
		return () => {
			cancelled = true;
		};
	}, []);

	const setThemeConfig = useCallback(
		async (nextConfig: ThemeConfig) => {
			const normalizedConfig = normalizeThemeSettings(nextConfig);
			const previousTheme = themeConfig;
			setThemeConfigState(normalizedConfig);
			try {
				const result = await commands.setTheme({
					themeKind: normalizedConfig.themeKind,
					themeId: normalizedConfig.themeId,
					accentColor: normalizedConfig.accentColor,
					colorMode: normalizedConfig.colorMode,
				});
				if (result.status === "error") {
					throw new Error(result.error.message);
				}
				setThemeConfigState(normalizeThemeSettings(result.data));
			} catch (error) {
				setThemeConfigState(previousTheme);
				throw error;
			}
		},
		[themeConfig],
	);

	return (
		<ThemeContext.Provider
			value={{
				themeKind: themeConfig.themeKind,
				themeId: themeConfig.themeId,
				accentColor: themeConfig.accentColor,
				colorMode: themeConfig.colorMode,
				effectiveMode,
				setThemeConfig,
			}}
		>
			{children}
		</ThemeContext.Provider>
	);
}

function normalizeThemeSettings(raw: Partial<SettingsSnapshotDto> | Partial<ThemeConfig>): ThemeConfig {
	const themeKind = raw.themeKind ?? DEFAULT_THEME_KIND;
	const themeId = raw.themeId ?? DEFAULT_THEME_ID;
	const accentColor = raw.accentColor ?? DEFAULT_ACCENT_COLOR;
	const colorMode = raw.colorMode ?? DEFAULT_COLOR_MODE;
	if (themeKind === "catppuccin") {
		return {
			themeKind,
			themeId: CATPPUCCIN_THEMES.has(themeId) ? themeId : "mocha",
			accentColor: null,
			colorMode,
		};
	}
	return {
		themeKind: "heroui",
		themeId: "default",
		accentColor,
		colorMode,
	};
}

export function useTheme() {
	const context = useContext(ThemeContext);
	if (!context) throw new Error("useTheme must be used within ThemeProvider");
	return context;
}
