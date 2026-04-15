import {
	Button,
	Card,
	Label,
	ListBox,
	NumberField,
	Select,
	Switch,
} from "@heroui/react";
import { useEffect, useMemo } from "react";
import { commands } from "../bindings";
import { useDashboardRuntime } from "../features/dashboard/runtime";
import { useDraftState } from "../features/dashboard/useDraftState";

function toInt(value: string, fallback = 0): number {
	const parsed = Number.parseInt(value, 10);
	return Number.isFinite(parsed) ? parsed : fallback;
}

export function AdvancedPage() {
	const { snapshot, busyAction, runDashboardAction } = useDashboardRuntime();

	const animeEnabled = useDraftState(false);
	const animeBrightness = useDraftState(0);
	const screenpadBrightness = useDraftState(0);
	const screenpadGamma = useDraftState(1);
	const syncScreenpad = useDraftState(false);
	const scsiEnabled = useDraftState(false);
	const scsiMode = useDraftState(0);
	const armouryPath = useDraftState<string | null>(null);
	const armouryValue = useDraftState(0);

	useEffect(() => {
		if (!snapshot) {
			return;
		}
		if (snapshot.anime.enableDisplay != null) {
			animeEnabled.syncFromSnapshot(snapshot.anime.enableDisplay);
		}
		if (snapshot.anime.brightness != null) {
			animeBrightness.syncFromSnapshot(toInt(snapshot.anime.brightness, 0));
		}
		if (snapshot.backlight.screenpadBrightness != null) {
			screenpadBrightness.syncFromSnapshot(snapshot.backlight.screenpadBrightness);
		}
		if (snapshot.backlight.screenpadGamma != null) {
			screenpadGamma.syncFromSnapshot(
				Number.parseFloat(snapshot.backlight.screenpadGamma) || 0,
			);
		}
		if (snapshot.backlight.syncScreenpadBrightness != null) {
			syncScreenpad.syncFromSnapshot(snapshot.backlight.syncScreenpadBrightness);
		}
		if (snapshot.scsi.enabled != null) {
			scsiEnabled.syncFromSnapshot(snapshot.scsi.enabled);
		}
		if (snapshot.scsi.mode != null) {
			scsiMode.syncFromSnapshot(toInt(snapshot.scsi.mode, 0));
		}
		const armouryAttributes = snapshot.armoury.attributes;
		if (armouryAttributes.length === 0) {
			armouryPath.reset(null);
			armouryValue.reset(0);
		} else if (
			!armouryPath.value ||
			!armouryAttributes.some((attribute) => attribute.path === armouryPath.value)
		) {
			const first = armouryAttributes[0];
			armouryPath.reset(first.path);
			armouryValue.reset(first.currentValue ?? first.defaultValue ?? 0);
		}
	}, [
		snapshot,
		animeEnabled,
		animeBrightness,
		screenpadBrightness,
		screenpadGamma,
		syncScreenpad,
		scsiEnabled,
		scsiMode,
		armouryPath,
		armouryValue,
	]);

	const animeEnabledAvailable =
		snapshot?.interfaces.asusdAnimeAvailable ?? false;
	const backlightAvailable =
		snapshot?.interfaces.asusdBacklightAvailable ?? false;
	const scsiAvailable = snapshot?.interfaces.asusdScsiAvailable ?? false;
	const armouryAvailable = snapshot?.interfaces.asusdArmouryAvailable ?? false;

	const selectedArmoury = useMemo(
		() =>
			snapshot?.armoury.attributes.find((item) => item.path === armouryPath.value) ??
			null,
		[snapshot?.armoury.attributes, armouryPath.value],
	);

	useEffect(() => {
		if (selectedArmoury?.currentValue != null) {
			armouryValue.syncFromSnapshot(selectedArmoury.currentValue);
		}
	}, [selectedArmoury?.currentValue, armouryValue]);

	const screenpadBrightnessValid =
		Number.isFinite(screenpadBrightness.value) &&
		Number.isInteger(screenpadBrightness.value);
	const screenpadGammaValid = Number.isFinite(screenpadGamma.value);
	const scsiModeValid =
		Number.isFinite(scsiMode.value) && Number.isInteger(scsiMode.value);
	const armouryInputDisabled =
		!armouryAvailable || !!busyAction || selectedArmoury == null;

	return (
		<div className="space-y-6">
			<div className="grid gap-6 md:grid-cols-2">
				<Card className="dashboard-card p-6">
					<div className="space-y-5">
						<h3 className="text-lg font-bold">Anime Matrix</h3>
						<Switch
							isSelected={animeEnabled.value}
							onChange={animeEnabled.setFromUser}
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
							value={animeBrightness.value}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									animeBrightness.setFromUser(Math.round(value));
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
									void (async () => {
										const result = await runDashboardAction(
											"setAnimeDisplayEnabled",
											() =>
												commands.setAnimeDisplayEnabled({
													enabled: animeEnabled.value,
												}),
										);
										if (result) {
											animeEnabled.markClean();
										}
									})()
								}
							>
								Apply Display
							</Button>
							<Button
								isDisabled={!animeEnabledAvailable || !!busyAction}
								onPress={() =>
									void (async () => {
										const result = await runDashboardAction(
											"setAnimeBrightness",
											() =>
												commands.setAnimeBrightness({
													level: animeBrightness.value,
												}),
										);
										if (result) {
											animeBrightness.markClean();
										}
									})()
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
							value={screenpadBrightness.value}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									screenpadBrightness.setFromUser(Math.round(value));
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
							value={screenpadGamma.value}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									screenpadGamma.setFromUser(value);
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
							isSelected={syncScreenpad.value}
							onChange={syncScreenpad.setFromUser}
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
								void (async () => {
									const result = await runDashboardAction("setBacklight", () =>
									commands.setBacklight({
										screenpadBrightness: screenpadBrightness.value,
										screenpadGamma: screenpadGamma.value,
										syncScreenpadBrightness: syncScreenpad.value,
									}),
									);
									if (result) {
										screenpadBrightness.markClean();
										screenpadGamma.markClean();
										syncScreenpad.markClean();
									}
								})()
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
							isSelected={scsiEnabled.value}
							onChange={scsiEnabled.setFromUser}
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
							value={scsiMode.value}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									scsiMode.setFromUser(Math.round(value));
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
									void (async () => {
										const result = await runDashboardAction(
											"setScsiEnabled",
											() =>
												commands.setScsiEnabled({
													enabled: scsiEnabled.value,
												}),
										);
										if (result) {
											scsiEnabled.markClean();
										}
									})()
								}
							>
								Apply Enabled
							</Button>
							<Button
								isDisabled={!scsiAvailable || !!busyAction || !scsiModeValid}
								onPress={() =>
									void (async () => {
										const result = await runDashboardAction("setScsiMode", () =>
											commands.setScsiMode({ mode: scsiMode.value }),
										);
										if (result) {
											scsiMode.markClean();
										}
									})()
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
							selectedKey={armouryPath.value}
							onSelectionChange={(key) => {
								if (key !== null) {
									armouryPath.setFromUser(String(key));
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
								selectedKeys={armouryPath.value ? [armouryPath.value] : []}
								selectionMode="single"
								onSelectionChange={(keys) => {
									const key = Array.from(keys)[0];
									if (key) {
										armouryPath.setFromUser(String(key));
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
							value={armouryValue.value}
							onChange={(value) => {
								if (Number.isFinite(value)) {
									armouryValue.setFromUser(Math.round(value));
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
								void (async () => {
									const result = await runDashboardAction("setArmouryValue", () =>
									commands.setArmouryValue({
										path: selectedArmoury?.path ?? "",
										value: Number.isFinite(armouryValue.value)
											? armouryValue.value
											: selectedArmoury?.currentValue ?? 0,
									}),
									);
									if (result) {
										armouryPath.markClean();
										armouryValue.markClean();
									}
								})()
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
