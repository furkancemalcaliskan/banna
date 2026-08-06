import { useFormik } from 'formik';
import i18n from '../../utils/i18n';
import {
  Box,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  Input,
  InputField,
  VStack,
} from '@gluestack-ui/themed';
import React, { useRef } from 'react';
import { KeyboardAvoidingView, Platform, TextInput } from 'react-native';
import * as Yup from 'yup';
import { FormButtonBar } from '../../components/FormButtons';
import ValidationMessage from '../../components/ValidationMessage/ValidationMessage';
import { useInputColors } from '../../theme';

type ManageProfileFormProps = {
  editingUser?: Record<string, any>;
  submit: (data: Record<string, any>) => void;
  cancel: () => void;
};

const ValidationSchema = Yup.object().shape({
  userName: Yup.string().required(i18n.t('AbpAccount::ThisFieldIsRequired')),
  email: Yup.string()
    .required(i18n.t('AbpAccount::ThisFieldIsRequired'))
    .email(i18n.t('AbpAccount::ThisFieldIsNotAValidEmailAddress')),
});

function ManageProfileForm({
  editingUser = {},
  submit,
  cancel,
}: ManageProfileFormProps) {
  const usernameRef = useRef<TextInput | null>(null);
  const nameRef = useRef<TextInput | null>(null);
  const surnameRef = useRef<TextInput | null>(null);
  const emailRef = useRef<TextInput | null>(null);
  const phoneNumberRef = useRef<TextInput | null>(null);
  const inputColors = useInputColors();

  const onSubmit = (values: Record<string, any>) => {
    submit({
      ...editingUser,
      ...values,
    });
  };

  const formik = useFormik({
    enableReinitialize: true,
    validationSchema: ValidationSchema,
    initialValues: {
      ...editingUser,
    },
    onSubmit,
  });

  return (
    <>
      <Box px="$3" pb={96}>
        <KeyboardAvoidingView
          behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
        >
          <FormControl isRequired my="$2" isInvalid={!!formik.errors.userName}>
            <VStack mx="$4" space="xs">
              <FormControlLabel>
                <FormControlLabelText>
                  {i18n.t('AbpIdentity::UserName')}
                </FormControlLabelText>
              </FormControlLabel>
              <Input
                bg={inputColors.bg}
                borderWidth={1}
                borderColor={inputColors.borderColor}
                borderRadius="$lg"
              >
                <InputField
                  ref={usernameRef}
                  onSubmitEditing={() => nameRef.current?.focus()}
                  returnKeyType="next"
                  onChangeText={formik.handleChange('userName')}
                  onBlur={formik.handleBlur('userName')}
                  value={formik.values.userName}
                  color={inputColors.textColor}
                  placeholderTextColor={inputColors.placeholderColor}
                  selectionColor={inputColors.textColor}
                />
              </Input>
              <ValidationMessage>{formik.errors.userName}</ValidationMessage>
            </VStack>
          </FormControl>

          <FormControl my="$2">
            <VStack mx="$4" space="xs">
              <FormControlLabel>
                <FormControlLabelText>
                  {i18n.t('AbpIdentity::DisplayName:Name')}
                </FormControlLabelText>
              </FormControlLabel>
              <Input
                bg={inputColors.bg}
                borderWidth={1}
                borderColor={inputColors.borderColor}
                borderRadius="$lg"
              >
                <InputField
                  ref={nameRef}
                  onSubmitEditing={() => surnameRef.current?.focus()}
                  returnKeyType="next"
                  onChangeText={formik.handleChange('name')}
                  onBlur={formik.handleBlur('name')}
                  value={formik.values.name}
                  color={inputColors.textColor}
                  placeholderTextColor={inputColors.placeholderColor}
                  selectionColor={inputColors.textColor}
                />
              </Input>
            </VStack>
          </FormControl>

          <FormControl my="$2">
            <VStack mx="$4" space="xs">
              <FormControlLabel>
                <FormControlLabelText>
                  {i18n.t('AbpIdentity::DisplayName:Surname')}
                </FormControlLabelText>
              </FormControlLabel>
              <Input
                bg={inputColors.bg}
                borderWidth={1}
                borderColor={inputColors.borderColor}
                borderRadius="$lg"
              >
                <InputField
                  ref={surnameRef}
                  onSubmitEditing={() => phoneNumberRef.current?.focus()}
                  returnKeyType="next"
                  onChangeText={formik.handleChange('surname')}
                  onBlur={formik.handleBlur('surname')}
                  value={formik.values.surname}
                  color={inputColors.textColor}
                  placeholderTextColor={inputColors.placeholderColor}
                  selectionColor={inputColors.textColor}
                />
              </Input>
            </VStack>
          </FormControl>

          <FormControl my="$2">
            <VStack mx="$4" space="xs">
              <FormControlLabel>
                <FormControlLabelText>
                  {i18n.t('AbpIdentity::PhoneNumber')}
                </FormControlLabelText>
              </FormControlLabel>
              <Input
                bg={inputColors.bg}
                borderWidth={1}
                borderColor={inputColors.borderColor}
                borderRadius="$lg"
              >
                <InputField
                  ref={phoneNumberRef}
                  onSubmitEditing={() => emailRef.current?.focus()}
                  returnKeyType="next"
                  onChangeText={formik.handleChange('phoneNumber')}
                  onBlur={formik.handleBlur('phoneNumber')}
                  value={formik.values.phoneNumber}
                  color={inputColors.textColor}
                  placeholderTextColor={inputColors.placeholderColor}
                  selectionColor={inputColors.textColor}
                />
              </Input>
            </VStack>
          </FormControl>

          <FormControl isRequired my="$2" isInvalid={!!formik.errors.email}>
            <VStack mx="$4" space="xs">
              <FormControlLabel>
                <FormControlLabelText>
                  {i18n.t('AbpIdentity::EmailAddress')}
                </FormControlLabelText>
              </FormControlLabel>
              <Input
                bg={inputColors.bg}
                borderWidth={1}
                borderColor={inputColors.borderColor}
                borderRadius="$lg"
              >
                <InputField
                  ref={emailRef}
                  returnKeyType="done"
                  onChangeText={formik.handleChange('email')}
                  onBlur={formik.handleBlur('email')}
                  value={formik.values.email}
                  color={inputColors.textColor}
                  placeholderTextColor={inputColors.placeholderColor}
                  selectionColor={inputColors.textColor}
                />
              </Input>
              <ValidationMessage>{formik.errors.email}</ValidationMessage>
            </VStack>
          </FormControl>
        </KeyboardAvoidingView>
      </Box>
      <FormButtonBar
        submit={formik.handleSubmit}
        cancel={cancel}
        isSubmitDisabled={!formik.isValid}
      />
    </>
  );
}

export default ManageProfileForm;
