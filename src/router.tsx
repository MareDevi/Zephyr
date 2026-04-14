import {
	createRootRoute,
	createRoute,
	createRouter,
	Outlet,
} from "@tanstack/react-router";
import { DashboardRuntimeProvider } from "./features/dashboard/runtime";
import { AppShell } from "./layout/AppShell";
import { AdvancedPage } from "./routes/ControlsPage";
import { FanCurvesPage } from "./routes/FanCurvesPage";
import { LightingPage } from "./routes/LightingPage";
import { OverviewPage } from "./routes/OverviewPage";
import { ProfilesPage } from "./routes/ProfilesPage";

function RootLayout() {
	return (
		<DashboardRuntimeProvider>
			<AppShell>
				<Outlet />
			</AppShell>
		</DashboardRuntimeProvider>
	);
}

const rootRoute = createRootRoute({
	component: RootLayout,
});

const overviewRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/",
	component: OverviewPage,
});

const profilesRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/profiles",
	component: ProfilesPage,
});

const fanCurvesRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/fancurves",
	component: FanCurvesPage,
});

const auraRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/aura",
	component: LightingPage,
});

const advancedRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/advanced",
	component: AdvancedPage,
});

const routeTree = rootRoute.addChildren([
	overviewRoute,
	profilesRoute,
	fanCurvesRoute,
	auraRoute,
	advancedRoute,
]);

export const router = createRouter({
	routeTree,
});

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}
