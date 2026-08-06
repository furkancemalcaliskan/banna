import { useFormik } from 'formik';
import i18n from '../../utils/i18n';
import {
  Box,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  Input,
  InputField,
  InputSlot,
  Icon,
  VStack,
} from '@gluestack-ui/themed';
import { useInputColors } from '../../theme';
import React, { useRef, useState } from 'react';
import * as Yup from 'yup';
import { FormButtonBar } from '../../components/FormButtons';
import ValidationMessage from '../../components/ValidationMessage/ValidationMessage';
import { Ionicons } from '@expo/vector-icons';
import { TextInput } from 'react-native';

const ValidationSchema = Yup.object().shape({
  currentPassword: Yup.string().required(
    i18n.t('AbpAccount::ThisFieldIsRequired'),
  ),
  newPassword: Yup.string().required(i18n.t('AbpAccount::ThisFieldIsRequired')),
});

type ChangePasswordFormProps = {
  submit: (data: Record<string, any>) => void;
  cancel: () => void;
};

function ChangePasswordForm({ submit, cancel }: ChangePasswordFormProps) {
  const [showCurrentPassword, setShowCurrentPassword] = useState(false);
  const [showNewPassword, setShowNewPassword] = useState(false);
  const inputColors = useInputColors();

  const currentPasswordRef = useRef<TextInput | null>(null);
  const newPasswordRef = useRef<TextInput | null>(null);

  const onSubmit = (values: Record<string, any>) => {
    submit({
      ...values,
      newPasswordConfirm: values.newPassword,
    });
  };

  const formik = useFormik({
    enableReinitialize: true,
    validationSchema: ValidationSchema,
    initialValues: {
      currentPassword: '',
      newPassword: '',
    },
    onSubmit,
  });

  return (
    <>
      <Box px="$3" pb={96}>
        <FormControl
          isRequired
          my="$2"
          isInvalid={!!formik.errors.currentPassword}
        >
          <VStack mx="$4" space="xs">
          <FormControlLabel>
            <FormControlLabelText>
              {i18n.t('AbpIdentity::DisplayName:CurrentPassword')}
            </FormControlLabelText>
          </FormControlLabel>
          <Input
            isInvalid={
              !!(
                formik.errors.currentPassword &&
                formik.touched.currentPassword
              )
            }
            borderWidth={1}
            borderRadius="$lg"
            bg={inputColors.bg}
            borderColor={inputColors.borderColor}
          >
            <InputField
              ref={currentPasswordRef}
              onSubmitEditing={() => newPasswordRef?.current?.focus()}
              returnKeyType="next"
              placeholder={i18n.t('AbpAccount::DisplayName:CurrentPassword')}
              onChangeText={formik.handleChange('currentPassword')}
              onBlur={formik.handleBlur('currentPassword')}
              value={formik.values.currentPassword}
              textContentType="password"
              type={showCurrentPassword ? 'text' : 'password'}
              autoCapitalize="none"
              color={inputColors.textColor}
              placeholderTextColor={inputColors.placeholderColor}
              selectionColor={inputColors.textColor}
            />
            <InputSlot
              pr="$3"
              onPress={() => setShowCurrentPassword(!showCurrentPassword)}
            >
              <Icon
                as={Ionicons}
                name={showCurrentPassword ? 'eye-off-outline' : 'eye-outline'}
                size="xl"
                color={inputColors.iconColor}
              />
            </InputSlot>
          </Input>
          <ValidationMessage>
            {formik.errors.currentPassword}
            </ValidationMessage>
          </VStack>
        </FormControl>

        <FormControl isRequired my="$2" isInvalid={!!formik.errors.newPassword}>
          <VStack mx="$4" space="xs">
            <FormControlLabel>
              <FormControlLabelText>
                {i18n.t('AbpIdentity::DisplayName:NewPassword')}
            </FormControlLabelText>
          </FormControlLabel>
          <Input
            isInvalid={
              !!(formik.errors.newPassword && formik.touched.newPassword)
            }
            borderWidth={1}
            borderRadius="$lg"
            bg={inputColors.bg}
            borderColor={inputColors.borderColor}
          >
            <InputField
              ref={newPasswordRef}
              returnKeyType="done"
              placeholder={i18n.t('AbpAccount::DisplayName:NewPassword')}
              onChangeText={formik.handleChange('newPassword')}
              onBlur={formik.handleBlur('newPassword')}
              value={formik.values.newPassword}
              textContentType="newPassword"
              type={showNewPassword ? 'text' : 'password'}
              autoCapitalize="none"
              color={inputColors.textColor}
              placeholderTextColor={inputColors.placeholderColor}
              selectionColor={inputColors.textColor}
            />
            <InputSlot
              pr="$3"
              onPress={() => setShowNewPassword(!showNewPassword)}
            >
              <Icon
                as={Ionicons}
                name={showNewPassword ? 'eye-off-outline' : 'eye-outline'}
                size="xl"
                color={inputColors.iconColor}
              />
            </InputSlot>
          </Input>
          <ValidationMessage>{formik.errors.newPassword}</ValidationMessage>
        </VStack>
        </FormControl>
      </Box>
      <FormButtonBar
        submit={formik.handleSubmit}
        cancel={cancel}
        isSubmitDisabled={!formik.isValid}
      />
    </>
  );
}

export default ChangePasswordForm;
