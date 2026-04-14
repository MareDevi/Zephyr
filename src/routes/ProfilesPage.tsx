import {
	Button,
	Card,
	Checkbox,
	Label,
	ListBox,
	Select,
	Slider,
	Tabs,
} from "@heroui/react";
import { useEffect, useMemo, useState } from "react";
import { commands } from "../bindings";
import {
	extractOptions,
	useDashboardRuntime,
} from "../features/dashboard/runtime";

const PLATFORM_PROFILE_FALLBACK = [
	"balanced",
	"performance",
	"quiet",
	"low-power",
	"custom",
];

function PlatformProfileTab() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();
	const [selectedProfile, setSelectedProfile] = useState("balanced");
	const [setOnAc, setSetOnAc] = useState(false);
	const [setOnBattery, setSetOnBattery] = useState(false);

	const profileOptions = useMemo(
		() =>
			extractOptions(
				snapshot?.platform.platformProfileChoices,
				PLATFORM_PROFILE_FALLBACK,
			),
		[snapshot?.platform.platformProfileChoices],
	);

	useEffect(() => {
		if (snapshot?.platform.platformProfile) {
			setSelectedProfile(snapshot.platform.platformProfile);
		}
	}, [snapshot]);

	const platformControlEnabled =
		snapshot?.interfaces.asusdPlatformAvailable ?? false;
	const supportsAcProfileTarget = snapshot?.platform.platformProfileOnAc != null;
	const supportsBatteryProfileTarget =
		snapshot?.platform.platformProfileOnBattery != null;

	return (
		<div className="flex flex-col gap-6">
			<Card className="bg-content1 p-6 border-none shadow-none">
				<div className="flex flex-col gap-6">
					<div>
						<h3 className="text-lg font-bold">Platform Profile</h3>
						<p className="text-xs text-default-500 mt-1">
							Configure performance profiles provided by the ASUS daemon.
						</p>
					</div>

					<Select
						aria-label="Select Platform Profile"
						selectedKey={selectedProfile}
						onSelectionChange={(key) => setSelectedProfile(String(key))}
						isDisabled={!platformControlEnabled || !!busyAction}
					>
						<Select.Trigger>
							<Select.Value />
							<Select.Indicator />
						</Select.Trigger>
						<Select.Popover>
							<ListBox
								aria-label="Platform profile options"
								selectedKeys={[selectedProfile]}
								onSelectionChange={(keys) => {
									const key = Array.from(keys)[0];
									if (key) setSelectedProfile(String(key));
								}}
							>
								{profileOptions.map((profile) => (
									<ListBox.Item key={profile} id={profile} textValue={profile}>
										{profile}
										<ListBox.ItemIndicator />
									</ListBox.Item>
								))}
							</ListBox>
						</Select.Popover>
					</Select>

					<div className="flex flex-col gap-3">
						<Checkbox
							isSelected={setOnAc}
							onChange={setSetOnAc}
							isDisabled={
								!platformControlEnabled || !!busyAction || !supportsAcProfileTarget
							}
						>
							<Checkbox.Control />
							<Checkbox.Content>
								<Label className="text-sm">Apply to AC profile</Label>
							</Checkbox.Content>
						</Checkbox>
						<Checkbox
							isSelected={setOnBattery}
							onChange={setSetOnBattery}
							isDisabled={
								!platformControlEnabled ||
								!!busyAction ||
								!supportsBatteryProfileTarget
							}
						>
							<Checkbox.Control />
							<Checkbox.Content>
								<Label className="text-sm">Apply to battery profile</Label>
							</Checkbox.Content>
						</Checkbox>
					</div>

					<Button
						variant="primary"
						className="font-bold"
						isDisabled={!platformControlEnabled || !!busyAction}
						onPress={() =>
							void runDashboardAction("setPlatformProfile", () =>
								commands.setPlatformProfile({
									profile: selectedProfile,
									ac: supportsAcProfileTarget ? setOnAc : undefined,
									battery: supportsBatteryProfileTarget
										? setOnBattery
										: undefined,
								}),
							)
						}
					>
						Set Profile
					</Button>
				</div>
			</Card>
		</div>
	);
}

function GpuProfileTab() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();
	const [selectedGpuMode, setSelectedGpuMode] = useState("Hybrid");

	const gpuModes = useMemo(() => {
		const parsed = extractOptions(snapshot?.gpu.supportedModes, []);
		if (parsed.length > 0) {
			return parsed;
		}
		if (snapshot?.gpu.mode) {
			return [snapshot.gpu.mode];
		}
		return [];
	}, [snapshot?.gpu.supportedModes, snapshot?.gpu.mode]);

	useEffect(() => {
		if (snapshot?.gpu.mode) {
			setSelectedGpuMode(snapshot.gpu.mode);
		}
	}, [snapshot?.gpu.mode]);

	const gpuControlEnabled = snapshot?.interfaces.supergfxdInterfaceAvailable ?? false;

	return (
		<div className="flex flex-col gap-6">
			<Card className="bg-content1 p-6 border-none shadow-none">
				<div className="flex flex-col gap-6">
					<h3 className="text-lg font-bold">GPU Profile</h3>
					<p className="text-xs text-default-500">
						Select a supergfxd mode for GPU routing and power behavior.
					</p>

					<Select
						aria-label="Select GPU Mode"
						selectedKey={selectedGpuMode}
						onSelectionChange={(key) => setSelectedGpuMode(String(key))}
						isDisabled={!gpuControlEnabled || !gpuModes.length || !!busyAction}
					>
						<Select.Trigger>
							<Select.Value />
							<Select.Indicator />
						</Select.Trigger>
						<Select.Popover>
							<ListBox
								aria-label="GPU mode options"
								selectedKeys={[selectedGpuMode]}
								onSelectionChange={(keys) => {
									const key = Array.from(keys)[0];
									if (key) setSelectedGpuMode(String(key));
								}}
							>
								{gpuModes.map((mode) => (
									<ListBox.Item key={mode} id={mode} textValue={mode}>
										{mode}
										<ListBox.ItemIndicator />
									</ListBox.Item>
								))}
							</ListBox>
						</Select.Popover>
					</Select>

					{!gpuControlEnabled ? (
						<p className="text-xs text-warning">
							GPU controls are unavailable on this system.
						</p>
					) : null}

					<Button
						variant="primary"
						className="font-bold"
						isDisabled={!gpuControlEnabled || !gpuModes.length || !!busyAction}
						onPress={() =>
							void runDashboardAction("setGpuMode", () =>
								commands.setGpuMode({ mode: selectedGpuMode }),
							)
						}
					>
						Apply GPU Mode
					</Button>
				</div>
			</Card>
		</div>
	);
}

function PowerProfileTab() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();
	const [chargeLimit, setChargeLimit] = useState(80);

	useEffect(() => {
		if (snapshot?.platform.chargeControlEndThreshold != null) {
			setChargeLimit(snapshot.platform.chargeControlEndThreshold);
		}
	}, [snapshot]);

	const chargeControlEnabled =
		(snapshot?.interfaces.asusdPlatformAvailable ?? false) &&
		snapshot?.platform.chargeControlEndThreshold != null;

	return (
		<div className="flex flex-col gap-6">
			<Card className="bg-content1 p-6 border-none shadow-none">
				<div className="flex flex-col gap-6">
					<div>
						<h3 className="text-lg font-bold">Charge Limit</h3>
						<p className="text-xs text-default-500 mt-1">
							Protect your battery health by limiting the maximum charge level.
						</p>
					</div>

					<div className="space-y-8">
						<Slider
							aria-label="Charge Limit"
							minValue={20}
							maxValue={100}
							step={1}
							value={chargeLimit}
							onChange={(val) => setChargeLimit(val as number)}
							isDisabled={!chargeControlEnabled || !!busyAction}
						>
							<Slider.Output className="text-xl font-bold" />
							<Slider.Track className="h-2">
								<Slider.Fill />
								<Slider.Thumb />
							</Slider.Track>
						</Slider>

						<div className="rounded-xl bg-default-100 p-4 text-xs text-default-500">
							<p className="font-semibold text-foreground mb-1">
								Battery Health Tip
							</p>
							Setting a limit below 80% can significantly extend the long-term
							lifespan of your laptop battery.
						</div>
					</div>

					<Button
						variant="primary"
						className="font-bold"
						isDisabled={!chargeControlEnabled || !!busyAction}
						onPress={() =>
							void runDashboardAction("setChargeLimit", () =>
								commands.setChargeLimit({
									percent: chargeLimit,
								}),
							)
						}
					>
						Update Charge Policy
					</Button>
				</div>
			</Card>
		</div>
	);
}

export function ProfilesPage() {
	return (
		<div className="space-y-6">
			<Tabs aria-label="Profile tabs" variant="primary">
				<Tabs.List>
					<Tabs.Tab id="platform">Platform</Tabs.Tab>
					<Tabs.Tab id="gpu">GPU</Tabs.Tab>
					<Tabs.Tab id="power">Power</Tabs.Tab>
				</Tabs.List>
				<Tabs.Panel id="platform">
					<PlatformProfileTab />
				</Tabs.Panel>
				<Tabs.Panel id="gpu">
					<GpuProfileTab />
				</Tabs.Panel>
				<Tabs.Panel id="power">
					<PowerProfileTab />
				</Tabs.Panel>
			</Tabs>
		</div>
	);
}
