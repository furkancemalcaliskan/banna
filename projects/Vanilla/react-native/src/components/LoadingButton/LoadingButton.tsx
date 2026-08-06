import { Button, Spinner, ButtonText, HStack, ButtonProps } from '@gluestack-ui/themed';
import React from 'react';
import { StyleSheet } from 'react-native';

type LoadingButtonProps = ButtonProps & {
  loading?: boolean;
  children?: React.ReactNode;
};

const LoadingButton: React.FC<LoadingButtonProps> = ({ loading = false, children, ...props }) => {
  return (
    <Button style={styles.button} {...props} isDisabled={loading} borderRadius="$xl" boxShadow="$glow">
      <HStack space="sm" alignItems="center">
        <ButtonText>{children}</ButtonText>
        {loading ? <Spinner color="$white" /> : null}
      </HStack>
    </Button>
  );
};

const styles = StyleSheet.create({
  button: { marginTop: 20, marginBottom: 30, height: 40 },
});

export default LoadingButton;
