import {
	Button,
	Card,
	Input,
	Label,
	ListBox,
	Select,
	Slider,
	Switch,
} from "@heroui/react";
import { useEffect, useMemo, useState } from "react";
import { commands } from "../bindings";
import { useDashboardRuntime } from "../features/dashboard/runtime";

function toInt(value: string, fallback = 0): number {
	const parsed = Number.parseInt(value, 10);
	return Number.isFinite(parsed) ? parsed : fallback;
}

export function AdvancedPage() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();

	const [animeEnabled, setAnimeEnabled] = useState(false);
	const [animeBrightness, setAnimeBrightness] = useState(0);
	const [screenpadBrightness, setScreenpadBrightness] = useState("0");
	const [screenpadGamma, setScreenpadGamma] = useState("1");
	const [syncScreenpad, setSyncScreenpad] = useState(false);
	const [scsiEnabled, setScsiEnabled] = useState(false);
	const [scsiMode, setScsiMode] = useState("0");
	const [armouryPath, setArmouryPath] = useState<string | null>(null);
	const [armouryValue, setArmouryValue] = useState("0");

	useEffect(() => {
		if (!snapshot) {
			return;
		}
		if (snapshot.anime.enableDisplay != null) {
			setAnimeEnabled(snapshot.anime.enableDisplay);
		}
		if (snapshot.anime.brightness != null) {
			setAnimeBrightness(toInt(snapshot.anime.brightness, 0));
		}
		if (snapshot.backlight.screenpadBrightness != null) {
			setScreenpadBrightness(String(snapshot.backlight.screenpadBrightness));
		}
		if (snapshot.backlight.screenpadGamma != null) {
			setScreenpadGamma(String(snapshot.backlight.screenpadGamma));
		}
		if (snapshot.backlight.syncScreenpadBrightness != null) {
			setSyncScreenpad(snapshot.backlight.syncScreenpadBrightness);
		}
		if (snapshot.scsi.enabled != null) {
			setScsiEnabled(snapshot.scsi.enabled);
		}
		if (snapshot.scsi.mode != null) {
			setScsiMode(String(snapshot.scsi.mode));
		}
		const armouryAttributes = snapshot.armoury.attributes;
		if (armouryAttributes.length === 0) {
			setArmouryPath(null);
			setArmouryValue("0");
		} else if (
			!armouryPath ||
			!armouryAttributes.some((attribute) => attribute.path === armouryPath)
		) {
			const first = armouryAttributes[0];
			setArmouryPath(first.path);
			setArmouryValue(String(first.currentValue ?? first.defaultValue ?? 0));
		}
	}, [snapshot, armouryPath]);

	const animeEnabledAvailable =
		snapshot?.interfaces.asusdAnimeAvailable ?? false;
	const backlightAvailable =
		snapshot?.interfaces.asusdBacklightAvailable ?? false;
	const scsiAvailable = snapshot?.interfaces.asusdScsiAvailable ?? false;
	const armouryAvailable = snapshot?.interfaces.asusdArmouryAvailable ?? false;

	const selectedArmoury = useMemo(
		() =>
			snapshot?.armoury.attributes.find((item) => item.path === armouryPath) ??
			null,
		[snapshot?.armoury.attributes, armouryPath],
	);

	useEffect(() => {
		if (selectedArmoury?.currentValue != null) {
			setArmouryValue(String(selectedArmoury.currentValue));
		}
	}, [selectedArmoury?.currentValue]);

	const screenpadBrightnessValid =
		screenpadBrightness.trim() !== "" &&
		Number.isFinite(Number.parseInt(screenpadBrightness, 10));
	const screenpadGammaValid =
		screenpadGamma.trim() !== "" &&
		Number.isFinite(Number.parseFloat(screenpadGamma));
	const scsiModeValid =
		scsiMode.trim() !== "" && Number.isFinite(Number.parseInt(scsiMode, 10));
	const armouryInputDisabled =
		!armouryAvailable || !!busyAction || selectedArmoury == null;

	return (
		<div className="space-y-6">
			<div className="grid gap-6 md:grid-cols-2">
				<Card className="dashboard-card p-6">
					<div className="space-y-5">
						<h3 className="text-lg font-bold">Anime Matrix</h3>
						<Switch
							isSelected={animeEnabled}
							onChange={setAnimeEnabled}
							isDisabled={!animeEnabledAvailable || !!busyAction}
						>
							<Switch.Control>
								<Switch.Thumb />
							</Switch.Control>
							<Switch.Content>
								<Label>Enable Anime Display</Label>
							</Switch.Content>
						</Switch>
						<Slider
							minValue={0}
							maxValue={3}
							step={1}
							value={animeBrightness}
							onChange={(value) => setAnimeBrightness(value as number)}
							isDisabled={!animeEnabledAvailable || !!busyAction}
						>
							<Label>Brightness</Label>
							<Slider.Output />
							<Slider.Track>
								<Slider.Fill />
								<Slider.Thumb />
							</Slider.Track>
						</Slider>
						<div className="flex gap-2">
							<Button
								isDisabled={!animeEnabledAvailable || !!busyAction}
								onPress={() =>
									void runDashboardAction("setAnimeDisplayEnabled", () =>
										commands.setAnimeDisplayEnabled({ enabled: animeEnabled }),
									)
								}
							>
								Apply Display
							</Button>
							<Button
								isDisabled={!animeEnabledAvailable || !!busyAction}
								onPress={() =>
									void runDashboardAction("setAnimeBrightness", () =>
										commands.setAnimeBrightness({ level: animeBrightness }),
									)
								}
							>
								Apply Brightness
							</Button>
						</div>
					</div>
				</Card>

				<Card className="dashboard-card p-6">
					<div className="space-y-5">
						<h3 className="text-lg font-bold">Backlight</h3>
						<Label htmlFor="advanced-screenpad-brightness">
							ScreenPad Brightness
						</Label>
						<Input
							id="advanced-screenpad-brightness"
							type="number"
							value={screenpadBrightness}
							onChange={(event) => setScreenpadBrightness(event.target.value)}
							disabled={!backlightAvailable || !!busyAction}
						/>
						<Label htmlFor="advanced-screenpad-gamma">ScreenPad Gamma</Label>
						<Input
							id="advanced-screenpad-gamma"
							type="number"
							step="0.1"
							value={screenpadGamma}
							onChange={(event) => setScreenpadGamma(event.target.value)}
							disabled={!backlightAvailable || !!busyAction}
						/>
						<Switch
							isSelected={syncScreenpad}
							onChange={setSyncScreenpad}
							isDisabled={!backlightAvailable || !!busyAction}
						>
							<Switch.Control>
								<Switch.Thumb />
							</Switch.Control>
							<Switch.Content>
								<Label>Sync with keyboard brightness</Label>
							</Switch.Content>
						</Switch>
						<Button
							isDisabled={
								!backlightAvailable ||
								!!busyAction ||
								!screenpadBrightnessValid ||
								!screenpadGammaValid
							}
							onPress={() =>
								void runDashboardAction("setBacklight", () =>
									commands.setBacklight({
										screenpadBrightness: Number.parseInt(
											screenpadBrightness,
											10,
										),
										screenpadGamma: Number.parseFloat(screenpadGamma),
										syncScreenpadBrightness: syncScreenpad,
									}),
								)
							}
						>
							Apply Backlight
						</Button>
					</div>
				</Card>
			</div>

			<div className="grid gap-6 md:grid-cols-2">
				<Card className="dashboard-card p-6">
					<div className="space-y-5">
						<h3 className="text-lg font-bold">SCSI</h3>
						<Switch
							isSelected={scsiEnabled}
							onChange={setScsiEnabled}
							isDisabled={!scsiAvailable || !!busyAction}
						>
							<Switch.Control>
								<Switch.Thumb />
							</Switch.Control>
							<Switch.Content>
								<Label>Enable SCSI Device</Label>
							</Switch.Content>
						</Switch>
						<Label htmlFor="advanced-scsi-mode">Mode</Label>
						<Input
							id="advanced-scsi-mode"
							type="number"
							min={0}
							max={255}
							value={scsiMode}
							onChange={(event) => setScsiMode(event.target.value)}
							disabled={!scsiAvailable || !!busyAction}
						/>
						<div className="flex gap-2">
							<Button
								isDisabled={!scsiAvailable || !!busyAction}
								onPress={() =>
									void runDashboardAction("setScsiEnabled", () =>
										commands.setScsiEnabled({ enabled: scsiEnabled }),
									)
								}
							>
								Apply Enabled
							</Button>
							<Button
								isDisabled={!scsiAvailable || !!busyAction || !scsiModeValid}
								onPress={() =>
									void runDashboardAction("setScsiMode", () =>
										commands.setScsiMode({ mode: toInt(scsiMode, 0) }),
									)
								}
							>
								Apply Mode
							</Button>
						</div>
					</div>
				</Card>

				<Card className="dashboard-card p-6">
					<div className="space-y-5">
						<h3 className="text-lg font-bold">Armoury Attributes</h3>
						<Select
							placeholder="Select attribute"
							selectedKey={armouryPath}
							onSelectionChange={(key) => {
								if (key !== null) {
									setArmouryPath(String(key));
								}
							}}
							isDisabled={!armouryAvailable || !!busyAction}
						>
							<Label>Attribute Path</Label>
							<Select.Trigger>
								<Select.Value />
								<Select.Indicator />
							</Select.Trigger>
							<Select.Popover>
								<ListBox
									aria-label="Armoury attribute options"
									selectedKeys={armouryPath ? [armouryPath] : []}
									selectionMode="single"
									onSelectionChange={(keys) => {
										const key = Array.from(keys)[0];
										if (key) {
											setArmouryPath(String(key));
										}
									}}
								>
									{(snapshot?.armoury.attributes ?? []).map((attribute) => (
										<ListBox.Item
											key={attribute.path}
											id={attribute.path}
											textValue={attribute.path}
										>
											{attribute.path}
											<ListBox.ItemIndicator />
										</ListBox.Item>
									))}
								</ListBox>
							</Select.Popover>
						</Select>
						<Label htmlFor="advanced-armoury-value">Current Value</Label>
						<Input
							id="advanced-armoury-value"
							type="number"
							value={armouryValue}
							onChange={(event) => setArmouryValue(event.target.value)}
							disabled={armouryInputDisabled}
						/>
						<Button
							isDisabled={armouryInputDisabled}
							onPress={() =>
								void runDashboardAction("setArmouryValue", () =>
									commands.setArmouryValue({
										path: selectedArmoury?.path ?? "",
										value: toInt(
											armouryValue,
											selectedArmoury?.currentValue ?? 0,
										),
									}),
								)
							}
						>
							Apply Attribute
						</Button>
					</div>
				</Card>
			</div>
		</div>
	);
}
