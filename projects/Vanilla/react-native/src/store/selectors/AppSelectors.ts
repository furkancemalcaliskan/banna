import { createSelector } from 'reselect';
import {
  appLanguages,
  resolveLocale,
} from '../../localization/appLocalization';

const getApp = state => state.app;
const getPersistentStorage = state => state.persistentStorage;

export function createAppConfigSelector() {
  return createSelector([getApp], state => state.appConfig);
}

export function createLanguageSelector() {
  return createSelector([getPersistentStorage], persistentStorage => {
    const cultureName = resolveLocale(persistentStorage?.language);
    return (
      appLanguages.find(language => language.cultureName === cultureName) || {
        cultureName,
        displayName: cultureName,
      }
    );
  });
}

export function createLanguagesSelector() {
  return createSelector(
    [() => appLanguages],
    languages => [...languages],
  );
}

export function createGrantedPolicySelector(key) {
  return createSelector([getApp], state => state?.appConfig?.auth?.grantedPolicies[key] ?? false);
}
