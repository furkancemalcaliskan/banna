import { Spinner, Box, Center } from '@gluestack-ui/themed';
import React, { forwardRef } from 'react';
import { StyleSheet, StyleProp, ViewStyle } from 'react-native';
import {
  createLoadingSelector,
  createOpacitySelector
} from '../../store/selectors/LoadingSelectors';
import { connectToRedux } from '../../utils/ReduxConnect';
import { useAppColorMode } from '../../theme';

type LoadingProps = {
  style?: StyleProp<ViewStyle>;
  loading?: boolean;
  opacity?: number;
};

const Loading: React.FC<LoadingProps> = ({ loading, opacity }) => {
  const activeMode = useAppColorMode();
  const spinnerColor = activeMode === 'dark' ? '$primary200' : '$primary600';
  return loading ? (
    <Center style={StyleSheet.absoluteFill} zIndex={9999}>
      <Box
        style={StyleSheet.absoluteFill}
        bg="$themeSurfaceBg"
        _dark={{ bg: '$themeSurfaceBgDark' }}
        opacity={opacity || 0.6}
      />
      <Spinner size="large" color={spinnerColor} />
    </Center>
  ) : null;
};

const Forwarded = forwardRef<unknown, LoadingProps>((props, ref) => (
  <Loading {...props} forwardedRef={ref as any} />
));

export default connectToRedux({
  component: Forwarded,
  stateProps: state => ({
    loading: createLoadingSelector()(state),
    opacity: createOpacitySelector()(state),
  }),
});
