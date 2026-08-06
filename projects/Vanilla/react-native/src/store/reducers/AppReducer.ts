import { createReducer } from '@reduxjs/toolkit';
import AppActions from '../actions/AppActions';

type AppState = {
  appConfig: Record<string, any>;
};

const initialState: AppState = {
  appConfig: {},
};

export default createReducer(initialState, builder =>
  builder.addCase(AppActions.setAppConfig, (state, action) => {
    state.appConfig = action.payload;
  }),
);
