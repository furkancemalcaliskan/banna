import React, { useEffect, useState } from 'react';
import { createUser, getUserById, updateUser } from '../../api/IdentityAPI';
import LoadingActions from '../../store/actions/LoadingActions';
import { createLoadingSelector } from '../../store/selectors/LoadingSelectors';
import { connectToRedux } from '../../utils/ReduxConnect';
import CreateUpdateUserForm from './CreateUpdateUserForm';

type CreateUpdateUserScreenProps = {
  navigation: any;
  route: any;
  startLoading: (payload: any) => void;
  stopLoading: (payload: any) => void;
};

function CreateUpdateUserScreen({
  navigation,
  route,
  startLoading,
  stopLoading,
}: CreateUpdateUserScreenProps) {
  const [user, setUser] = useState<Record<string, any> | undefined>();
  const userId = route.params?.userId;

  useEffect(() => {
    if (userId) {
      getUserById(userId).then((data = {}) => setUser(data));
    }
  }, [userId]);

  const submit = (data: Record<string, any>) => {
    startLoading({ key: 'saveUser' });
    let request;
    if (data.id) {
      request = updateUser(data, userId);
    } else {
      request = createUser(data);
    }

    request
      .then(() => {
        navigation.goBack();
      })
      .finally(() => stopLoading({ key: 'saveUser' }));
  };

  const renderForm = () => (
    <CreateUpdateUserForm editingUser={user} submit={submit} />
  );

  if (userId && user) {
    return renderForm();
  }

  if (!userId) {
    return renderForm();
  }

  return null;
}

export default connectToRedux({
  component: CreateUpdateUserScreen,
  stateProps: state => ({ loading: createLoadingSelector()(state) }),
  dispatchProps: {
    startLoading: LoadingActions.start,
    stopLoading: LoadingActions.stop,
  },
});
