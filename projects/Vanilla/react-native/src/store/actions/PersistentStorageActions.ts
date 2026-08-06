import { createAction } from '@reduxjs/toolkit';

const setToken = createAction('persistentStorage/setToken');

const setLanguage = createAction('persistentStorage/setLanguage');

const setTenant = createAction('persistentStorage/setTenant');

const setColorMode = createAction('persistentStorage/setColorMode');

export default {
  setToken,
  setLanguage,
  setTenant,
  setColorMode,
};
