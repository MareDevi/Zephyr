import { Card, ProgressBar, Separator } from "@heroui/react";
import { useMemo } from "react";
import {
	Line,
	LineChart,
	CartesianGrid,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";
import { useDashboardRuntime } from "../features/dashboard/runtime";

function formatNumber(
	value: number | null | undefined,
	suffix: string,
): string {
	if (value == null) {
		return "N/A";
	}
	return `${value}${suffix}`;
}

function clampUsage(value: number | null | undefined): number {
	if (value == null || Number.isNaN(value)) {
		return 0;
	}
	return Math.min(100, Math.max(0, value));
}

export function OverviewPage() {
	const { snapshot, history } = useDashboardRuntime();
	const gpuModeDisplay =
		snapshot?.gpu.mode ??
		(snapshot?.performance.gpu.source === "nvml" ? "Nvidia" : "N/A");

	const cpuUsage = clampUsage(snapshot?.performance.cpu.utilizationPercent);
	const gpuUsage = clampUsage(snapshot?.performance.gpu.utilizationPercent);

	const chartData = useMemo(
		() =>
			history.map((point, index) => ({
				tick: index,
				cpu: clampUsage(point.cpu),
				gpu: clampUsage(point.gpu),
				ram: clampUsage(point.ram),
			})),
		[history],
	);

	return (
		<div className="space-y-8">
			<section className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
				<Card className="bg-content1 p-5 shadow-none border-none">
					<div className="flex flex-col gap-4">
						<span className="text-sm font-medium text-default-500">
							Platform Profile
						</span>
						<div className="flex flex-col">
							<span className="text-2xl font-bold text-foreground">
								{snapshot?.platform.platformProfile ?? "Standard"}
							</span>
							<span className="text-xs text-default-400 mt-1">
								{snapshot?.power.activeProfile ?? "Balanced"}
							</span>
						</div>
					</div>
				</Card>

				<Card className="bg-content1 p-5 shadow-none border-none">
					<div className="flex flex-col gap-4">
						<span className="text-sm font-medium text-default-500">
							GPU Mode
						</span>
						<div className="flex flex-col">
							<span className="text-2xl font-bold text-foreground">
								{gpuModeDisplay}
							</span>
							<span className="text-xs text-default-400 mt-1">
								{snapshot?.performance.gpu.source ?? "Discrete"}
							</span>
						</div>
					</div>
				</Card>

				<Card className="bg-content1 p-5 shadow-none border-none">
					<div className="flex flex-col gap-4">
						<span className="text-sm font-medium text-default-500">
							Power Profile
						</span>
						<div className="flex flex-col">
							<span className="text-2xl font-bold text-foreground">
								{snapshot?.power.activeProfile ?? "N/A"}
							</span>
							<span className="text-xs text-default-400 mt-1">
								{snapshot?.power.performanceDegraded ?? "No degradation"}
							</span>
						</div>
					</div>
				</Card>

				<Card className="bg-content1 p-5 shadow-none border-none">
					<div className="flex flex-col gap-4">
						<span className="text-sm font-medium text-default-500">
							Daemon Status
						</span>
						<div className="flex flex-col">
							<span className="text-2xl font-bold text-foreground">
								{snapshot?.health.asusdAvailable ? "Active" : "Offline"} /{" "}
								{snapshot?.health.supergfxdAvailable ? "GPU OK" : "GPU Off"}
							</span>
							<span className="text-xs text-default-400 mt-1">
								{snapshot?.health.ppdAvailable ? "PPD Online" : "PPD Offline"}
							</span>
						</div>
					</div>
				</Card>
			</section>

			<div className="grid gap-6 lg:grid-cols-12">
				<Card className="lg:col-span-8 bg-content1 border-none shadow-none overflow-hidden">
					<div className="p-6">
						<div className="flex items-center justify-between mb-8">
							<div className="flex flex-col">
						<h3 className="text-lg font-bold">Real-time Performance</h3>
								<div className="flex items-center gap-4 mt-1">
									<div className="flex items-center gap-2">
										<span className="text-2xl font-bold text-primary">
											{cpuUsage}%
										</span>
										<span className="text-[10px] font-bold text-success uppercase">
											Weekly avg.
										</span>
									</div>
									<Separator orientation="vertical" className="h-4" />
									<div className="flex items-center gap-2">
										<span className="text-2xl font-bold text-foreground">
											{formatNumber(
												snapshot?.performance.cpu.frequencyMhz,
												"M",
											)}
										</span>
										<span className="text-[10px] font-bold text-success uppercase">
											Freq avg.
										</span>
									</div>
								</div>
							</div>
						</div>

						<div className="h-64 w-full">
							<div className="mb-4 flex flex-wrap items-center gap-4 text-xs font-semibold">
								<span className="text-primary">CPU</span>
								<span className="text-success">GPU</span>
								<span className="text-warning">RAM</span>
							</div>
							<ResponsiveContainer width="100%" height="100%">
							<LineChart
								data={chartData}
								margin={{ left: -20, right: 0, top: 10, bottom: 0 }}
							>
								<CartesianGrid
									strokeDasharray="3 3"
									vertical={false}
										strokeOpacity={0.1}
									/>
									<XAxis dataKey="tick" hide />
									<YAxis domain={[0, 100]} hide />
									<Tooltip
										contentStyle={{
											backgroundColor: "var(--content1)",
											borderRadius: "12px",
											border: "1px solid var(--default-200)",
										}}
									itemStyle={{ color: "var(--primary)" }}
								/>
								<Line
									type="monotone"
									dataKey="cpu"
									stroke="var(--primary)"
									strokeWidth={2}
									dot={false}
								/>
								<Line
									type="monotone"
									dataKey="gpu"
									stroke="var(--success)"
									strokeWidth={2}
									dot={false}
								/>
								<Line
									type="monotone"
									dataKey="ram"
									stroke="var(--warning)"
									strokeWidth={2}
									dot={false}
								/>
							</LineChart>
						</ResponsiveContainer>
					</div>
				</div>
			</Card>

				<div className="lg:col-span-4 space-y-6">
					<Card className="bg-content1 p-6 border-none shadow-none">
						<div className="flex items-center justify-between mb-6">
							<h3 className="text-lg font-bold">GPU Analytics</h3>
						</div>
						<div className="space-y-6">
							<div className="flex flex-col gap-2">
								<span className="text-3xl font-bold text-foreground">
									{gpuUsage}%
								</span>
								<span className="text-xs text-default-500">Utilization</span>
							</div>

							<div className="h-32 w-full">
								<ResponsiveContainer width="100%" height="100%">
									<LineChart
										data={chartData}
										margin={{ left: 0, right: 0, top: 0, bottom: 0 }}
									>
										<Line
											type="step"
											dataKey="gpu"
											stroke="var(--success)"
											strokeWidth={2}
											dot={false}
										/>
									</LineChart>
								</ResponsiveContainer>
							</div>

							<div className="space-y-4">
								<div className="flex flex-col gap-2">
									<div className="flex justify-between text-xs font-medium">
										<span className="text-default-500">Clock Speed</span>
										<span>
											{formatNumber(
												snapshot?.performance.gpu.frequencyMhz,
												" MHz",
											)}
										</span>
									</div>
									<ProgressBar value={gpuUsage} color="success" size="sm" />
								</div>
								<div className="grid grid-cols-2 gap-3">
									<div className="bg-default-100 p-3 rounded-xl flex flex-col gap-1">
										<span className="text-[10px] font-bold text-default-400 uppercase">
											VRAM
										</span>
										<span className="text-sm font-bold">4.2 GB</span>
									</div>
									<div className="bg-default-100 p-3 rounded-xl flex flex-col gap-1">
										<span className="text-[10px] font-bold text-default-400 uppercase">
											Temp
										</span>
										<span className="text-sm font-bold">62 °C</span>
									</div>
								</div>
							</div>
						</div>
					</Card>
				</div>
			</div>
		</div>
	);
}
