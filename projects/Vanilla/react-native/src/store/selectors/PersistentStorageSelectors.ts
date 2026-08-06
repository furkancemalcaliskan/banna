import { createSelector } from 'reselect';

const getPersistentStorage = state => state.persistentStorage;

export function createTokenSelector() {
  return createSelector([getPersistentStorage], persistentStorage => persistentStorage.token);
}

export function createTenantSelector() {
  return createSelector([getPersistentStorage], persistentStorage => persistentStorage.tenant);
}

export function createColorModeSelector() {
  return createSelector([getPersistentStorage], persistentStorage => persistentStorage.colorMode || 'system');
}
