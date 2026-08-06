import { useFocusEffect } from '@react-navigation/native';
import React, { useCallback, useState } from 'react';
import {
  createTenant,
  getTenantById,
  updateTenant
} from '../../api/TenantManagementAPI';
import LoadingActions from '../../store/actions/LoadingActions';
import { createLoadingSelector } from '../../store/selectors/LoadingSelectors';
import { connectToRedux } from '../../utils/ReduxConnect';
import CreateUpdateTenantForm from './CreateUpdateTenantForm';

type CreateUpdateTenantScreenProps = {
  navigation: any;
  route: any;
  startLoading: (payload: any) => void;
  stopLoading: (payload: any) => void;
};

function CreateUpdateTenantScreen({
  navigation,
  route,
  startLoading,
  stopLoading,
}: CreateUpdateTenantScreenProps) {
  const [tenant, setTenant] = useState<Record<string, any> | undefined>();
  const tenantId = route.params?.tenantId;

  useFocusEffect(
    useCallback(() => {
      if (tenantId) {
        getTenantById(tenantId).then((data = {}) => setTenant(data));
      }
    }, [tenantId]),
  );

  const submit = (data: Record<string, any>) => {
    startLoading({ key: 'saveTenant' });
    let request;
    if (data.id) {
      request = updateTenant(data, tenantId);
    } else {
      request = createTenant(data);
    }

    request
      .then(() => {
        navigation.goBack();
      })
      .finally(() => stopLoading({ key: 'saveTenant' }));
  };

  const renderForm = () => (
    <CreateUpdateTenantForm
      editingTenant={tenant}
      submit={submit}
    />
  );

  if (tenantId && tenant) {
    return renderForm();
  }

  if (!tenantId) {
    return renderForm();
  }

  return null;
}

export default connectToRedux({
  component: CreateUpdateTenantScreen,
  stateProps: state => ({ loading: createLoadingSelector()(state) }),
  dispatchProps: {
    startLoading: LoadingActions.start,
    stopLoading: LoadingActions.stop,
  },
});
