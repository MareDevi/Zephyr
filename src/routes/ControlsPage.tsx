import {
	Button,
	Card,
	Label,
	ListBox,
	NumberField,
	Select,
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
	const [screenpadBrightness, setScreenpadBrightness] = useState(0);
	const [screenpadGamma, setScreenpadGamma] = useState(1);
	const [syncScreenpad, setSyncScreenpad] = useState(false);
	const [scsiEnabled, setScsiEnabled] = useState(false);
	const [scsiMode, setScsiMode] = useState(0);
	const [armouryPath, setArmouryPath] = useState<string | null>(null);
	const [armouryValue, setArmouryValue] = useState(0);

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
			setScreenpadBrightness(snapshot.backlight.screenpadBrightness);
		}
		if (snapshot.backlight.screenpadGamma != null) {
			setScreenpadGamma(Number.parseFloat(snapshot.backlight.screenpadGamma) || 0);
		}
		if (snapshot.backlight.syncScreenpadBrightness != null) {
			setSyncScreenpad(snapshot.backlight.syncScreenpadBrightness);
		}
		if (snapshot.scsi.enabled != null) {
			setScsiEnabled(snapshot.scsi.enabled);
		}
		if (snapshot.scsi.mode != null) {
			setScsiMode(toInt(snapshot.scsi.mode, 0));
		}
		const armouryAttributes = snapshot.armoury.attributes;
		if (armouryAttributes.length === 0) {
			setArmouryPath(null);
			setArmouryValue(0);
		} else if (
			!armouryPath ||
			!armouryAttributes.some((attribute) => attribute.path === armouryPath)
		) {
			const first = armouryAttributes[0];
			setArmouryPath(first.path);
			setArmouryValue(first.currentValue ?? first.defaultValue ?? 0);
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
			setArmouryValue(selectedArmoury.currentValue);
		}
	}, [selectedArmoury?.currentValue]);

	const screenpadBrightnessValid =
		Number.isFinite(screenpadBrightness) &&
		Number.isInteger(screenpadBrightness);
	const screenpadGammaValid = Number.isFinite(screenpadGamma);
	const scsiModeValid = Number.isFinite(scsiMode) && Number.isInteger(scsiMode);
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
						<NumberField
							minValue={0}
							maxValue={3}
							step={1}
							value={animeBrightness}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									setAnimeBrightness(Math.round(value));
								}
							}}
							isDisabled={!animeEnabledAvailable || !!busyAction}
						>
							<Label>Brightness</Label>
							<NumberField.Group>
								<NumberField.DecrementButton />
								<NumberField.Input />
								<NumberField.IncrementButton />
							</NumberField.Group>
						</NumberField>
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
						<NumberField
							id="advanced-screenpad-brightness"
							minValue={0}
							value={screenpadBrightness}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									setScreenpadBrightness(Math.round(value));
								}
							}}
							isDisabled={!backlightAvailable || !!busyAction}
						>
							<NumberField.Group>
								<NumberField.DecrementButton />
								<NumberField.Input />
								<NumberField.IncrementButton />
							</NumberField.Group>
						</NumberField>
						<Label htmlFor="advanced-screenpad-gamma">ScreenPad Gamma</Label>
						<NumberField
							id="advanced-screenpad-gamma"
							step={0.1}
							value={screenpadGamma}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									setScreenpadGamma(value);
								}
							}}
							isDisabled={!backlightAvailable || !!busyAction}
						>
							<NumberField.Group>
								<NumberField.DecrementButton />
								<NumberField.Input />
								<NumberField.IncrementButton />
							</NumberField.Group>
						</NumberField>
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
										screenpadBrightness,
										screenpadGamma,
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
						<NumberField
							id="advanced-scsi-mode"
							minValue={0}
							maxValue={255}
							value={scsiMode}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									setScsiMode(Math.round(value));
								}
							}}
							isDisabled={!scsiAvailable || !!busyAction}
						>
							<NumberField.Group>
								<NumberField.DecrementButton />
								<NumberField.Input />
								<NumberField.IncrementButton />
							</NumberField.Group>
						</NumberField>
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
										commands.setScsiMode({ mode: scsiMode }),
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
						<NumberField
							id="advanced-armoury-value"
							value={armouryValue}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									setArmouryValue(Math.round(value));
								}
							}}
							isDisabled={armouryInputDisabled}
						>
							<NumberField.Group>
								<NumberField.DecrementButton />
								<NumberField.Input />
								<NumberField.IncrementButton />
							</NumberField.Group>
						</NumberField>
						<Button
							isDisabled={armouryInputDisabled}
							onPress={() =>
								void runDashboardAction("setArmouryValue", () =>
									commands.setArmouryValue({
										path: selectedArmoury?.path ?? "",
										value: Number.isFinite(armouryValue)
											? armouryValue
											: selectedArmoury?.currentValue ?? 0,
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
