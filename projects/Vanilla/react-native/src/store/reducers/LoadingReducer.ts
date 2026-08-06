import { createReducer } from '@reduxjs/toolkit';
import LoadingActions from '../actions/LoadingActions';

type LoadingState = {
  activeLoadings: Record<string, any>;
  loading: boolean;
  opacity: number;
};

const initialState: LoadingState = { activeLoadings: {}, loading: false, opacity: 1 };

export default createReducer(initialState, builder =>
  builder
    .addCase(LoadingActions.start, (state, action) => {
      const { key, opacity } = action.payload;
      state.activeLoadings[key] = action;
      state.loading = true;
      state.opacity = opacity ?? state.opacity;
    })
    .addCase(LoadingActions.stop, (state, action) => {
      if (state.activeLoadings[action.payload.key]) {
        delete state.activeLoadings[action.payload.key];
      }
      state.loading = Object.keys(state.activeLoadings || {}).length > 0;
    })
    .addCase(LoadingActions.clear, () => ({ ...initialState })),
);
