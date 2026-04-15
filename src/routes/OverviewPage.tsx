import { Card, ProgressBar, Separator } from "@heroui/react";
import { useMemo } from "react";
import {
	CartesianGrid,
	Line,
	LineChart,
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

type MetricCardProps = {
	label: string;
	value: string;
	hint: string;
};

function MetricCard({ label, value, hint }: MetricCardProps) {
	return (
		<Card className="dashboard-card p-5">
			<Card.Header className="flex-col items-start gap-1 p-0">
				<p className="text-xs font-semibold uppercase tracking-wider text-default-500">
					{label}
				</p>
				<p className="text-2xl font-bold tracking-tight">{value}</p>
			</Card.Header>
			<div className="p-0 pt-2">
				<p className="text-xs text-default-400">{hint}</p>
			</div>
		</Card>
	);
}

export function OverviewPage() {
	const { snapshot, history } = useDashboardRuntime();
	const gpuModeDisplay =
		snapshot?.gpu.mode ??
		(snapshot?.performance.gpu.source === "nvml" ? "Nvidia" : "N/A");

	const cpuUsage = clampUsage(snapshot?.performance.cpu.utilizationPercent);
	const gpuUsage = clampUsage(snapshot?.performance.gpu.utilizationPercent);
	const ramUsage = clampUsage(snapshot?.performance.ram.utilizationPercent);

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
		<div className="space-y-6">
			<section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
				<MetricCard
					label="Platform Profile"
					value={snapshot?.platform.platformProfile ?? "Standard"}
					hint={snapshot?.power.activeProfile ?? "Balanced"}
				/>
				<MetricCard
					label="GPU Mode"
					value={gpuModeDisplay}
					hint={snapshot?.performance.gpu.source ?? "Discrete source"}
				/>
				<MetricCard
					label="Power Profile"
					value={snapshot?.power.activeProfile ?? "N/A"}
					hint={snapshot?.power.performanceDegraded ?? "No degradation"}
				/>
				<MetricCard
					label="Daemon Health"
					value={snapshot?.health.asusdAvailable ? "Active" : "Offline"}
					hint={snapshot?.health.ppdAvailable ? "PPD online" : "PPD offline"}
				/>
			</section>

			<section className="grid gap-6 xl:grid-cols-12">
				<Card className="dashboard-card xl:col-span-8">
					<Card.Header className="flex-col items-start gap-2 p-6">
						<p className="section-title">Realtime performance</p>
						<div className="flex flex-wrap items-center gap-4 text-xs font-semibold">
							<span className="text-accent">CPU {cpuUsage}%</span>
							<Separator orientation="vertical" className="h-3" />
							<span className="text-success">GPU {gpuUsage}%</span>
							<Separator orientation="vertical" className="h-3" />
							<span className="text-warning">RAM {ramUsage}%</span>
						</div>
					</Card.Header>
					<div className="p-6 pt-0">
						<div className="h-72 w-full">
							<ResponsiveContainer width="100%" height="100%">
								<LineChart
									data={chartData}
									margin={{ left: -20, right: 0, top: 10, bottom: 0 }}
								>
									<CartesianGrid
										strokeDasharray="3 3"
										vertical={false}
										strokeOpacity={0.16}
									/>
									<XAxis dataKey="tick" hide />
									<YAxis domain={[0, 100]} hide />
									<Tooltip
										contentStyle={{
											backgroundColor: "var(--content1)",
											borderRadius: "12px",
											border: "1px solid var(--default-200)",
										}}
									/>
									<Line
										type="monotone"
										dataKey="cpu"
										stroke="var(--accent)"
										strokeWidth={2.4}
										dot={false}
									/>
									<Line
										type="monotone"
										dataKey="gpu"
										stroke="var(--success)"
										strokeWidth={2.4}
										dot={false}
									/>
									<Line
										type="monotone"
										dataKey="ram"
										stroke="var(--warning)"
										strokeWidth={2.4}
										dot={false}
									/>
								</LineChart>
							</ResponsiveContainer>
						</div>
					</div>
				</Card>

				<Card className="dashboard-card xl:col-span-4">
					<Card.Header className="flex-col items-start gap-1 p-6">
						<p className="section-title">GPU analytics</p>
						<p className="section-description">
							Realtime utilization and effective clock stability.
						</p>
					</Card.Header>
					<div className="space-y-5 p-6 pt-0">
						<div className="space-y-2">
							<div className="flex items-center justify-between text-sm">
								<span className="text-default-500">Utilization</span>
								<span className="font-semibold">{gpuUsage}%</span>
							</div>
							<ProgressBar value={gpuUsage} color="success" size="sm" />
						</div>

						<div className="space-y-2">
							<div className="flex items-center justify-between text-sm">
								<span className="text-default-500">Clock</span>
								<span className="font-semibold">
									{formatNumber(snapshot?.performance.gpu.frequencyMhz, " MHz")}
								</span>
							</div>
							<ProgressBar
								value={Math.min(100, gpuUsage + 6)}
								color="accent"
								size="sm"
							/>
						</div>

						<div className="grid grid-cols-2 gap-3">
							<div className="rounded-xl border border-default-200 bg-default-100 p-3">
								<p className="text-[10px] uppercase tracking-wide text-default-500">
									VRAM
								</p>
								<p className="text-sm font-semibold">4.2 GB</p>
							</div>
							<div className="rounded-xl border border-default-200 bg-default-100 p-3">
								<p className="text-[10px] uppercase tracking-wide text-default-500">
									Temp
								</p>
								<p className="text-sm font-semibold">62 °C</p>
							</div>
						</div>
					</div>
				</Card>
			</section>
		</div>
	);
}
