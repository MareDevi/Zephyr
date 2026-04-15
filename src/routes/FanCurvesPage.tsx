import {
	Button,
	Card,
	Label,
	ListBox,
	NumberField,
	Select,
	Separator,
} from "@heroui/react";
import { IconRefresh, IconTrash, IconWind } from "@tabler/icons-react";
import { useEffect, useMemo, useState } from "react";
import {
	CartesianGrid,
	LabelList,
	Line,
	LineChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";
import { commands, type FanCurveSeriesSnapshot } from "../bindings";
import { useDashboardRuntime } from "../features/dashboard/runtime";

type CurvePoint = { temperature: number; pwm: number };

function parseLegacyCurve(
	raw: string | null | undefined,
): FanCurveSeriesSnapshot[] {
	if (!raw) {
		return [];
	}
	return raw
		.split("\n")
		.map((line) => {
			const fan = line.split(":")[0]?.trim() || "Unknown";
			const pwmMatch = line.match(/pwm=\[([^\]]+)\]/i);
			const tempMatch = line.match(/temp=\[([^\]]+)\]/i);
			if (!pwmMatch || !tempMatch) {
				return null;
			}
			const pwmValues = pwmMatch[1]
				.split(",")
				.map((item) => Number.parseInt(item.trim(), 10))
				.filter((item) => Number.isFinite(item));
			const tempValues = tempMatch[1]
				.split(",")
				.map((item) => Number.parseInt(item.trim(), 10))
				.filter((item) => Number.isFinite(item));
			const points = tempValues
				.slice(0, Math.min(tempValues.length, pwmValues.length))
				.map((temperature, index) => ({
					temperature,
					pwm: pwmValues[index],
				}));
			return {
				fan,
				enabled: /enabled=true/i.test(line),
				points,
			} satisfies FanCurveSeriesSnapshot;
		})
		.filter((item): item is FanCurveSeriesSnapshot => item !== null);
}

export function FanCurvesPage() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();
	const [fanProfile, setFanProfile] = useState("balanced");
	const [selectedFan, setSelectedFan] = useState<string | null>(null);
	const [editablePoints, setEditablePoints] = useState<CurvePoint[]>([]);

	const fanControlEnabled =
		snapshot?.interfaces.asusdFanCurvesAvailable ?? false;

	useEffect(() => {
		if (snapshot?.fanCurves.activeProfile) {
			setFanProfile(snapshot.fanCurves.activeProfile);
		}
	}, [snapshot?.fanCurves.activeProfile]);

	const series = useMemo(() => {
		const structured = snapshot?.fanCurves.curveSeries ?? [];
		if (structured.length > 0) {
			return structured;
		}
		return parseLegacyCurve(snapshot?.fanCurves.curveData);
	}, [snapshot?.fanCurves.curveSeries, snapshot?.fanCurves.curveData]);

	useEffect(() => {
		const fanNames = series.map((entry) => entry.fan);
		const firstFan = fanNames[0] ?? null;
		setSelectedFan((current) =>
			current && fanNames.includes(current) ? current : firstFan,
		);
	}, [series]);

	const visibleSeries =
		series.find((item) => item.fan === selectedFan) ?? series[0] ?? null;
	const points: CurvePoint[] = useMemo(
		() =>
			visibleSeries?.points.map((point) => ({
				temperature: point.temperature,
				pwm: point.pwm,
			})) ?? [],
		[visibleSeries],
	);

	useEffect(() => {
		setEditablePoints(points);
	}, [points]);

	const hasEditablePoints = editablePoints.length > 0;
	const hasLocalEdits = useMemo(
		() => JSON.stringify(editablePoints) !== JSON.stringify(points),
		[editablePoints, points],
	);
	const backendSaveAvailable =
		fanControlEnabled &&
		!!selectedFan &&
		hasEditablePoints &&
		hasLocalEdits &&
		!busyAction;

	return (
		<div className="space-y-8">
			<Card className="dashboard-card p-6">
				<div className="flex flex-col gap-6">
					<div className="flex items-center justify-between">
						<div>
							<h2 className="text-xl font-bold">Fan Curves</h2>
							<p className="text-xs text-default-500 mt-1">
								Configure custom fan speed curves for system cooling.
							</p>
						</div>
						<div
							className={`rounded-full px-3 py-1 text-[10px] font-bold uppercase ${
								fanControlEnabled
									? "bg-default text-success"
									: "bg-default text-danger"
							}`}
						>
							{fanControlEnabled ? "Available" : "Unavailable"}
						</div>
					</div>

					<div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
						<Select
							placeholder="Select Profile"
							selectedKey={fanProfile}
							onSelectionChange={(key) => setFanProfile(String(key))}
							isDisabled={!fanControlEnabled || !!busyAction}
						>
							<Label>Profile</Label>
							<Select.Trigger>
								<Select.Value />
								<Select.Indicator />
							</Select.Trigger>
							<Select.Popover>
								<ListBox
									aria-label="Fan profile options"
									selectedKeys={[fanProfile]}
									onSelectionChange={(keys) => {
										const key = Array.from(keys)[0];
										if (key) setFanProfile(String(key));
									}}
								>
									{[
										"balanced",
										"performance",
										"quiet",
										"low-power",
										"custom",
									].map((profile) => (
										<ListBox.Item
											key={profile}
											id={profile}
											textValue={profile}
										>
											{profile}
											<ListBox.ItemIndicator />
										</ListBox.Item>
									))}
								</ListBox>
							</Select.Popover>
						</Select>

						<div className="flex flex-col gap-2">
							<Button
								className="font-bold gap-2"
								isDisabled={!fanControlEnabled || !!busyAction}
								onPress={() =>
									void runDashboardAction("readFanCurves", () =>
										commands.readFanCurves({ profile: fanProfile }),
									)
								}
							>
								<IconRefresh size={18} />
								Read Curves
							</Button>
							<Button
								className="font-bold gap-2"
								isDisabled={!fanControlEnabled || !!busyAction}
								onPress={() =>
									void runDashboardAction("resetFanCurves", () =>
										commands.resetFanCurves({ profile: fanProfile }),
									)
								}
							>
								<IconTrash size={18} />
								Reset Default
							</Button>
						</div>

						<div className="flex items-end">
							<Button
								className="w-full font-bold gap-2"
								isDisabled={!fanControlEnabled || !!busyAction}
								onPress={() =>
									void runDashboardAction("setFanCurvesEnabled", () =>
										commands.setFanCurvesEnabled({
											profile: fanProfile,
											enabled: true,
										}),
									)
								}
							>
								<IconWind size={18} />
								Apply Custom
							</Button>
						</div>
					</div>
				</div>
			</Card>

			<Card className="dashboard-card overflow-hidden">
				<div className="p-6">
					<div className="flex items-center justify-between mb-6">
						<div className="flex gap-2 p-1 bg-default-100 rounded-xl">
							{series.map((entry) => (
								<Button
									key={entry.fan}
									size="sm"
									className={`rounded-lg font-bold text-xs ${
										selectedFan === entry.fan
											? "bg-content3 shadow-sm"
											: "text-default-500"
									}`}
									onPress={() => setSelectedFan(entry.fan)}
								>
									{entry.fan}
								</Button>
							))}
						</div>
					</div>

					{visibleSeries ? (
						<div className="h-80 w-full rounded-xl border border-default-100 p-4">
							<ResponsiveContainer width="100%" height="100%">
								<LineChart data={editablePoints}>
									<CartesianGrid
										strokeDasharray="3 3"
										vertical={false}
										strokeOpacity={0.1}
									/>
									<XAxis
										type="number"
										dataKey="temperature"
										domain={[0, 100]}
										stroke="var(--default-500)"
										fontSize={10}
										tickLine={false}
										axisLine={false}
									/>
									<YAxis
										type="number"
										dataKey="pwm"
										domain={[0, 100]}
										stroke="var(--default-500)"
										fontSize={10}
										tickLine={false}
										axisLine={false}
									/>
									<Tooltip
										contentStyle={{
											backgroundColor: "var(--content1)",
											borderRadius: "12px",
											border: "1px solid var(--default-200)",
										}}
										labelFormatter={(value) => `${value}°C`}
									/>
									<Line
										type="linear"
										dataKey="pwm"
										stroke="var(--accent)"
										strokeWidth={3}
										dot={{
											r: 4,
											fill: "var(--accent)",
											stroke: "var(--content1)",
											strokeWidth: 1.5,
										}}
										activeDot={{ r: 6, fill: "var(--accent)" }}
										isAnimationActive
									>
										<LabelList
											dataKey="pwm"
											position="top"
											formatter={(value) => `${value}%`}
											fontSize={10}
											fill="var(--foreground)"
										/>
									</Line>
								</LineChart>
							</ResponsiveContainer>
						</div>
					) : (
						<div className="rounded-xl border border-default-200 bg-default-50 p-4 text-sm text-default-600">
							No fan curve data loaded for this profile. Use{" "}
							<span className="font-semibold">Read Curves</span> to fetch
							current points.
						</div>
					)}

					<Separator className="my-6" />

					<div className="space-y-3">
						<div className="flex items-center justify-between">
							<Label className="text-sm font-semibold">Curve Editor</Label>
							<Button
								size="sm"
								isDisabled={!hasEditablePoints || !hasLocalEdits}
								onPress={() => setEditablePoints(points)}
							>
								Reset Local Edits
							</Button>
						</div>
						<div className="grid gap-3 md:grid-cols-2">
							{editablePoints.map((point, index) => (
								<div
									key={`${selectedFan ?? "fan"}-${point.temperature}-${point.pwm}`}
									className="grid grid-cols-2 gap-2 rounded-lg border border-default-100 p-3"
								>
									<div className="space-y-1">
										<NumberField
											minValue={0}
											maxValue={100}
											value={point.temperature}
											onChange={(nextValue) => {
												if (!Number.isFinite(nextValue)) {
													return;
												}
												setEditablePoints((current) =>
													current.map((entry, entryIndex) =>
														entryIndex === index
															? {
																	...entry,
																	temperature: Math.max(0, Math.min(100, Math.round(nextValue))),
																}
															: entry,
													),
												);
											}}
										>
											<Label>Temp (°C)</Label>
											<NumberField.Group>
												<NumberField.DecrementButton />
												<NumberField.Input />
												<NumberField.IncrementButton />
											</NumberField.Group>
										</NumberField>
									</div>
									<div className="space-y-1">
										<NumberField
											minValue={0}
											maxValue={100}
											value={point.pwm}
											onChange={(nextValue) => {
												if (!Number.isFinite(nextValue)) {
													return;
												}
												setEditablePoints((current) =>
													current.map((entry, entryIndex) =>
														entryIndex === index
															? {
																	...entry,
																	pwm: Math.max(0, Math.min(100, Math.round(nextValue))),
																}
															: entry,
													),
												);
											}}
										>
											<Label>PWM (%)</Label>
											<NumberField.Group>
												<NumberField.DecrementButton />
												<NumberField.Input />
												<NumberField.IncrementButton />
											</NumberField.Group>
										</NumberField>
									</div>
								</div>
							))}
						</div>
						<Button
							isDisabled={!backendSaveAvailable}
							className="font-semibold"
							onPress={() => {
								if (!selectedFan) {
									return;
								}
								const payloadPoints = editablePoints.map((point) => ({
									temperature: Math.max(
										0,
										Math.min(100, Math.round(point.temperature)),
									),
									pwm: Math.max(0, Math.min(100, Math.round(point.pwm))),
								}));
								void runDashboardAction("setFanCurve", () =>
									commands.setFanCurve({
										profile: fanProfile,
										fan: selectedFan,
										points: payloadPoints,
										enabled: visibleSeries?.enabled ?? true,
									}),
								);
							}}
						>
							Save to backend
						</Button>
						<p className="text-xs text-default-500">
							Edits are saved for the currently selected fan and active profile.
						</p>
						<span className="text-[10px] font-bold text-default-400 uppercase tracking-wider">
							Raw Curve Points
						</span>
						<pre className="max-h-36 overflow-auto rounded-xl bg-default-100 p-4 text-[10px] font-mono leading-relaxed">
							{snapshot?.fanCurves.curveData ??
								"No raw data available for the current profile."}
						</pre>
					</div>
				</div>
			</Card>
		</div>
	);
}
