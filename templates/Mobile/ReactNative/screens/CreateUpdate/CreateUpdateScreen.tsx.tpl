import { useEffect, useState } from 'react';

import { get, create, update } from '../../../api/__entity_name__API';
import LoadingActions from '../../../store/actions/LoadingActions';
import { createLoadingSelector } from '../../../store/selectors/LoadingSelectors';
import { connectToRedux } from '../../../utils/ReduxConnect';
import CreateUpdate__entity_name__Form from './CreateUpdate__entity_name__Form';

__mobile_screen_extra_imports__

type CreateUpdate__entity_name__ScreenProps = {
  navigation: any;
  route: any;
  startLoading: (payload: any) => void;
  clearLoading: (payload?: any) => void;
__mobile_screen_props__
};

function CreateUpdate__entity_name__Screen({
  navigation,
  route,
  startLoading,
  clearLoading,
__mobile_screen_props__
}: CreateUpdate__entity_name__ScreenProps) {
  const { __entity_name_lower__Id } = route.params || {};
  const [__entity_name_lower__, set__entity_name__] = useState<Record<string, any> | null>(null);
  const [refreshing, setRefreshing] = useState(false);
__mobile_screen_state__

  const submit = (data) => {
    startLoading({ key: 'save' });

    (data.id ? update(data, data.id) : create(data))
      .then(() => navigation.goBack())
      .finally(() => clearLoading());
  };

  const loadEntity = () => {
    if (!__entity_name_lower__Id) {
      return Promise.resolve();
    }

    startLoading({ key: 'fetch__entity_name__Detail' });

    return get(__entity_name_lower__Id)
      .then((response) => set__entity_name__(response?.__entity_name_lower__ ?? response))
      .finally(() => clearLoading());
  };

__mobile_screen_lookup_loaders__

  const loadLookups = () =>
    Promise.all([
__mobile_screen_lookup_calls__
    ]);

  const refresh = () => {
    setRefreshing(true);
    return Promise.all([loadEntity(), loadLookups()]).finally(() => setRefreshing(false));
  };

  useEffect(() => {
    loadEntity();
  }, [__entity_name_lower__Id]);

__mobile_screen_effects__

  return (
    <CreateUpdate__entity_name__Form
      submit={submit}
      __entity_name_lower__={__entity_name_lower__}
      refreshing={refreshing}
      onRefresh={refresh}
__mobile_screen_form_props__
    />
  );
}

export default connectToRedux({
  component: CreateUpdate__entity_name__Screen,
  stateProps: (state) => ({ loading: createLoadingSelector()(state) }),
  dispatchProps: {
    startLoading: LoadingActions.start,
    clearLoading: LoadingActions.clear,
  },
});
