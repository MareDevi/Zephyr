import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ReactNode } from "react";
import { useEffect } from "react";
import { create } from "zustand";
import { commands, type DashboardSnapshotDto } from "../../bindings";

const DASHBOARD_UPDATED_EVENT = "backend://dashboard-updated";
const HISTORY_LIMIT = 120;

type CommandResult<T> =
	| { status: "ok"; data: T }
	| { status: "error"; error: unknown };

type RuntimeSeriesPoint = {
	ts: number;
	cpu: number | null;
	gpu: number | null;
	ram: number | null;
};

type DashboardRuntimeState = {
	snapshot: DashboardSnapshotDto | null;
	error: string | null;
	lastActionError: { action: string; message: string; at: number } | null;
	busyAction: string | null;
	history: RuntimeSeriesPoint[];
	refresh: () => Promise<void>;
	runDashboardAction: (
		actionName: string,
		action: () => Promise<CommandResult<DashboardSnapshotDto>>,
	) => Promise<DashboardSnapshotDto | null>;
	setSnapshot: (snapshot: DashboardSnapshotDto) => void;
	setError: (message: string | null) => void;
};

function toErrorMessage(error: unknown): string {
	if (typeof error === "string") {
		return error;
	}
	if (error && typeof error === "object" && "message" in error) {
		const message = (error as { message: unknown }).message;
		if (typeof message === "string") {
			return message;
		}
	}
	return JSON.stringify(error);
}

function unwrapCommandResult<T>(result: CommandResult<T>): T {
	if (result.status === "ok") {
		return result.data;
	}
	throw new Error(toErrorMessage(result.error));
}

function toSeriesPoint(snapshot: DashboardSnapshotDto): RuntimeSeriesPoint {
	return {
		ts: snapshot.updatedAtMs,
		cpu: snapshot.performance.cpu.utilizationPercent,
		gpu: snapshot.performance.gpu.utilizationPercent,
		ram: snapshot.performance.ram.utilizationPercent,
	};
}

const useDashboardStore = create<DashboardRuntimeState>((set, get) => ({
	snapshot: null,
	error: null,
	lastActionError: null,
	busyAction: null,
	history: [],
	setSnapshot: (snapshot) =>
		set((state) => ({
			snapshot,
			error: null,
			history: [...state.history, toSeriesPoint(snapshot)].slice(-HISTORY_LIMIT),
		})),
	setError: (message) => set({ error: message }),
	runDashboardAction: async (actionName, action) => {
		try {
			set({ busyAction: actionName });
			const data = unwrapCommandResult(await action());
			get().setSnapshot(data);
			return data;
		} catch (error) {
			const message = toErrorMessage(error);
			set({
				error: message,
				lastActionError: { action: actionName, message, at: Date.now() },
			});
			return null;
		} finally {
			set({ busyAction: null });
		}
	},
	refresh: async () => {
		await get().runDashboardAction("refreshDashboardSnapshot", () =>
			commands.refreshDashboardSnapshot(),
		);
	},
}));

let runtimeInitialized = false;
let unlistenDashboard: UnlistenFn | null = null;

async function initDashboardRuntime() {
	if (runtimeInitialized) {
		return;
	}
	runtimeInitialized = true;

	try {
		const snapshot = unwrapCommandResult(await commands.getDashboardSnapshot());
		useDashboardStore.getState().setSnapshot(snapshot);
	} catch (error) {
		useDashboardStore.getState().setError(toErrorMessage(error));
	}

	try {
		unlistenDashboard = await listen<DashboardSnapshotDto>(
			DASHBOARD_UPDATED_EVENT,
			(event) => {
				useDashboardStore.getState().setSnapshot(event.payload);
			},
		);
	} catch (error) {
		useDashboardStore.getState().setError(toErrorMessage(error));
	}
}

function useDashboardBootstrap() {
	useEffect(() => {
		void initDashboardRuntime();
	}, []);
}

export function DashboardRuntimeProvider({ children }: { children: ReactNode }) {
	useDashboardBootstrap();
	return <>{children}</>;
}

export function useDashboardRuntime() {
	useDashboardBootstrap();
	return useDashboardStore();
}

export function cleanupDashboardRuntime() {
	if (unlistenDashboard) {
		unlistenDashboard();
		unlistenDashboard = null;
	}
	runtimeInitialized = false;
}

export function extractOptions(
	raw: string | null | undefined,
	fallback: string[],
): string[] {
	if (!raw) {
		return fallback;
	}
	const matches = raw.match(/[A-Za-z][A-Za-z0-9-]*/g) ?? [];
	const unique = Array.from(new Set(matches));
	return unique.length > 0 ? unique : fallback;
}
