/**
 * Test setup file for Vitest.
 *
 * This file is executed before each test file.
 * Use it to set up global mocks and test utilities.
 */

import "@testing-library/svelte/vitest";

// Mock window.matchMedia for components that use media queries
Object.defineProperty(window, "matchMedia", {
	writable: true,
	value: (query: string) => ({
		matches: false,
		media: query,
		onchange: null,
		addListener: () => {},
		removeListener: () => {},
		addEventListener: () => {},
		removeEventListener: () => {},
		dispatchEvent: () => false,
	}),
});

// Mock IntersectionObserver for lazy-loading components
class MockIntersectionObserver implements IntersectionObserver {
	readonly root: Element | null = null;
	readonly rootMargin: string = "";
	readonly thresholds: ReadonlyArray<number> = [];

	disconnect() {}
	observe() {}
	takeRecords(): IntersectionObserverEntry[] {
		return [];
	}
	unobserve() {}
}

Object.defineProperty(window, "IntersectionObserver", {
	writable: true,
	value: MockIntersectionObserver,
});

// Mock ResizeObserver for responsive components
class MockResizeObserver implements ResizeObserver {
	disconnect() {}
	observe() {}
	unobserve() {}
}

Object.defineProperty(window, "ResizeObserver", {
	writable: true,
	value: MockResizeObserver,
});
