import { expect, afterEach, vi } from 'vitest';

//
// Mock some methods used in the app, not provided by jsom
//
window.alert = vi.fn();
// Used by maplibre-gl
global.URL.createObjectURL = vi.fn();
global.URL.revokeObjectURL = vi.fn();