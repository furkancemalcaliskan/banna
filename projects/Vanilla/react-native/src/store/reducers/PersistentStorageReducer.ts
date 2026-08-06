import { createReducer } from '@reduxjs/toolkit';
import PersistentStorageActions from '../actions/PersistentStorageActions';

type PersistentStorageState = {
  token: Record<string, any>;
  language: string | null;
  tenant: Record<string, any>;
  colorMode: string;
};

const initialState: PersistentStorageState = { token: {}, language: null, tenant: {}, colorMode: 'system' };

export default createReducer(initialState, builder =>
  builder
    .addCase(PersistentStorageActions.setToken, (state, action) => {
      state.token = action.payload;
    })
    .addCase(PersistentStorageActions.setLanguage, (state, action) => {
      state.language = action.payload;
    })
    .addCase(PersistentStorageActions.setTenant, (state, action) => {
      state.tenant = action.payload;
    })
    .addCase(PersistentStorageActions.setColorMode, (state, action) => {
      state.colorMode = action.payload;
    }),
);
