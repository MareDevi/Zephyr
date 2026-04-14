import {
	createContext,
	type ReactNode,
	useContext,
	useEffect,
	useState,
} from "react";

export type ThemeMode = "light" | "dark";

export interface ColorScheme {
	id: string;
	name: string;
	accent: string; // OKLCH or Hex
}

export const PREDEFINED_SCHEMES: ColorScheme[] = [
	{ id: "default", name: "Default Blue", accent: "oklch(0.6204 0.195 253.83)" },
	{ id: "rog", name: "ROG Red", accent: "oklch(0.55 0.22 25)" },
	{ id: "tuf", name: "TUF Orange", accent: "oklch(0.65 0.2 45)" },
	{ id: "matrix", name: "Matrix Purple", accent: "oklch(0.6 0.22 300)" },
	{ id: "cyan", name: "Cyber Cyan", accent: "oklch(0.7 0.15 190)" },
	{ id: "emerald", name: "Emerald Green", accent: "oklch(0.65 0.2 150)" },
	{ id: "amber", name: "Amber Gold", accent: "oklch(0.75 0.18 70)" },
];

interface ThemeContextType {
	mode: ThemeMode;
	setMode: (mode: ThemeMode) => void;
	colorScheme: ColorScheme;
	setColorScheme: (scheme: ColorScheme) => void;
	setCustomColor: (color: string) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: ReactNode }) {
	const [mode, setMode] = useState<ThemeMode>(() => {
		const saved = localStorage.getItem("theme-mode");
		return (
			(saved as ThemeMode) ||
			(window.matchMedia("(prefers-color-scheme: dark)").matches
				? "dark"
				: "light")
		);
	});

	const [colorScheme, setColorScheme] = useState<ColorScheme>(() => {
		const savedId = localStorage.getItem("theme-scheme");
		const savedCustom = localStorage.getItem("theme-custom-color");
		if (savedId === "custom" && savedCustom) {
			return { id: "custom", name: "Custom Color", accent: savedCustom };
		}
		return (
			PREDEFINED_SCHEMES.find((s) => s.id === savedId) || PREDEFINED_SCHEMES[0]
		);
	});

	useEffect(() => {
		const root = document.documentElement;
		root.classList.remove("light", "dark");
		root.classList.add(mode);
		root.setAttribute("data-theme", mode);
		localStorage.setItem("theme-mode", mode);
	}, [mode]);

	useEffect(() => {
		const root = document.documentElement;
		root.style.setProperty("--accent", colorScheme.accent);
		localStorage.setItem("theme-scheme", colorScheme.id);
		if (colorScheme.id === "custom") {
			localStorage.setItem("theme-custom-color", colorScheme.accent);
		}
	}, [colorScheme]);

	const setCustomColor = (color: string) => {
		setColorScheme({ id: "custom", name: "Custom Color", accent: color });
	};

	return (
		<ThemeContext.Provider
			value={{ mode, setMode, colorScheme, setColorScheme, setCustomColor }}
		>
			{children}
		</ThemeContext.Provider>
	);
}

export function useTheme() {
	const context = useContext(ThemeContext);
	if (!context) throw new Error("useTheme must be used within ThemeProvider");
	return context;
}
