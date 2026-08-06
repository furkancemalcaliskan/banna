import React from 'react';
import { changePassword } from '../../api/IdentityAPI';
import LoadingActions from '../../store/actions/LoadingActions';
import { connectToRedux } from '../../utils/ReduxConnect';
import ChangePasswordForm from './ChangePasswordForm';

type ChangePasswordScreenProps = {
  navigation: any;
  startLoading: (payload: any) => void;
  stopLoading: (payload: any) => void;
};

function ChangePasswordScreen({
  navigation,
  startLoading,
  stopLoading,
}: ChangePasswordScreenProps) {
  const submit = data => {
    startLoading({ key: 'changePassword' });

    changePassword(data)
      .then(() => {
        navigation.goBack();
      })
      .finally(() => stopLoading({ key: 'changePassword' }));
  };

  return <ChangePasswordForm submit={submit} cancel={() => navigation.goBack()} />;
}

export default connectToRedux({
  component: ChangePasswordScreen,
  dispatchProps: {
    startLoading: LoadingActions.start,
    stopLoading: LoadingActions.stop,
  },
});
