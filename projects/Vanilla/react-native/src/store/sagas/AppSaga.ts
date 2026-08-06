import { all, call, put, takeLatest } from 'redux-saga/effects';
import { Logout } from '../../api/AccountAPI';
import { getApplicationConfiguration } from '../../api/ApplicationConfigurationAPI';
import AppActions from '../actions/AppActions';
import LoadingActions from '../actions/LoadingActions';
import PersistentStorageActions from '../actions/PersistentStorageActions';

function* fetchAppConfig({ payload } = {} as any) {
  const { showLoading = false, callback = () => {} } = payload || {};
  if (showLoading) {
    yield put(LoadingActions.start({ key: 'appConfig', opacity: 1 }));
  }

  const data = yield call(getApplicationConfiguration);
  yield put(AppActions.setAppConfig(data));
  if (showLoading) yield put(LoadingActions.stop({ key: 'appConfig' }));
  callback();
}

function* setLanguage(action: any) {
  yield put(PersistentStorageActions.setLanguage(action.payload));
}

function* logout({ payload: { client_id, token, refresh_token } }: any) {
  const data = { client_id, token };

  yield call(Logout, data);

  if (!!refresh_token) {
    data.token = refresh_token;
    data.token_type_hint = 'refresh_token';
    yield call(Logout, data);
  }

  yield put(PersistentStorageActions.setToken({}));
  yield put(AppActions.fetchAppConfigAsync({}));
}

export default function* () {
  yield all([
    takeLatest(AppActions.setLanguageAsync.type, setLanguage),
    takeLatest(AppActions.fetchAppConfigAsync.type, fetchAppConfig),
    takeLatest(AppActions.logoutAsync.type, logout),
  ]);
}
