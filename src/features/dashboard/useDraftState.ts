import { useCallback, useState, type SetStateAction } from "react";

export type DraftState<T> = {
	value: T;
	isDirty: boolean;
	setFromUser: (next: SetStateAction<T>) => void;
	syncFromSnapshot: (next: T) => void;
	reset: (next: T) => void;
	markClean: () => void;
};

export function useDraftState<T>(initialValue: T): DraftState<T> {
	const [value, setValue] = useState(initialValue);
	const [isDirty, setIsDirty] = useState(false);

	const setFromUser = useCallback((next: SetStateAction<T>) => {
		setIsDirty(true);
		setValue(next);
	}, []);

	const syncFromSnapshot = useCallback(
		(next: T) => {
			if (!isDirty) {
				setValue(next);
			}
		},
		[isDirty],
	);

	const reset = useCallback((next: T) => {
		setValue(next);
		setIsDirty(false);
	}, []);

	const markClean = useCallback(() => {
		setIsDirty(false);
	}, []);

	return {
		value,
		isDirty,
		setFromUser,
		syncFromSnapshot,
		reset,
		markClean,
	};
}
