import i18n from '../../utils/i18n';
import { Button, ButtonText, HStack, Box } from '@gluestack-ui/themed';
import React, { forwardRef } from 'react';
import { Alert, StyleSheet, StyleProp, ViewStyle } from 'react-native';

type FormButtonsProps = {
  submit: () => void;
  remove?: () => void;
  removeMessage?: string;
  isRemoveDisabled?: boolean;
  isSubmitDisabled?: boolean;
  isShowRemove?: boolean;
  isShowSubmit?: boolean;
  isFixed?: boolean;
  style?: StyleProp<ViewStyle>;
};

function FormButtons({
  submit,
  remove,
  removeMessage,
  isRemoveDisabled,
  isSubmitDisabled,
  isShowRemove = false,
  isShowSubmit = true,
  isFixed = true,
  style,
}: FormButtonsProps) {
  const confirmation = () => {
    Alert.alert(
      i18n.t('AbpUi::AreYouSure'),
      removeMessage,
      [
        {
          text: i18n.t('AbpUi::Cancel'),
          style: 'cancel',
        },
        { text: i18n.t('AbpUi::Yes'), onPress: () => remove() },
      ],
      { cancelable: true },
    );
  };

  const containerStyles = [
    styles.container,
    isFixed ? styles.fixed : styles.inline,
    style,
  ];

  return (
    <Box 
      bg="transparent"
      style={containerStyles}
    >
      <HStack width="100%" space="none">
        {isShowRemove ? (
          <Button
            action="negative"
            variant="solid"
            flex={1}
            borderRadius="$xl"
            boxShadow="$glow"
            _dark={{ boxShadow: "$glowDark" }}
            onPress={() => confirmation()}
            isDisabled={isRemoveDisabled}>
            <ButtonText>{i18n.t('AbpIdentity::Delete')}</ButtonText>
          </Button>
        ) : null}
        {isShowSubmit ? (
          <Button
            flex={1}
            borderRadius="$xl"
            boxShadow="$glow"
            _dark={{ boxShadow: "$glowDark" }}
            onPress={submit}
            isDisabled={isSubmitDisabled}>
            <ButtonText>{i18n.t('AbpIdentity::Save')}</ButtonText>
          </Button>
        ) : null}
      </HStack>
    </Box>
  );
}

const styles = StyleSheet.create({
  container: {
    width: '100%',
  },
  fixed: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
  },
  inline: {
    position: 'relative',
  },
});

const Forwarded = forwardRef<any, FormButtonsProps>((props, ref) => <FormButtons {...props} forwardedRef={ref} />);

export default Forwarded;
