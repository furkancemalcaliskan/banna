import { Ionicons } from '@expo/vector-icons';
import { useFormik } from 'formik';
import i18n from '../../utils/i18n';
import {
  Box,
  Button,
  ButtonText,
  Checkbox,
  CheckboxIndicator,
  CheckboxIcon,
  CheckboxLabel,
  CheckIcon,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  Icon,
  Input,
  InputField,
  InputSlot,
  VStack,
  HStack,
} from '@gluestack-ui/themed';
import React, { useRef, useState } from 'react';
import { KeyboardAvoidingView, Platform, TextInput } from 'react-native';
import * as Yup from 'yup';
import { FormButtonBar } from '../../components/FormButtons';
import ValidationMessage from '../../components/ValidationMessage/ValidationMessage';
import UserRoles from './UserRoles';

const validations = {
  userName: Yup.string().required(i18n.t('AbpAccount::ThisFieldIsRequired')),
  email: Yup.string()
    .email(i18n.t('AbpAccount::ThisFieldIsNotAValidEmailAddress'))
    .required(i18n.t('AbpAccount::ThisFieldIsRequired')),
};

let roleNames: string[] = [];

function onChangeRoles(roles: string[]) {
  roleNames = roles;
}

type CreateUpdateUserFormProps = {
  editingUser?: Record<string, any>;
  submit: (data: Record<string, any>) => void;
};

function CreateUpdateUserForm({
  editingUser = {},
  submit,
}: CreateUpdateUserFormProps) {
  const [selectedTab, setSelectedTab] = useState(0);
  const [showPassword, setShowPassword] = useState(false);

  const usernameRef = useRef<TextInput | null>(null);
  const nameRef = useRef<TextInput | null>(null);
  const surnameRef = useRef<TextInput | null>(null);
  const emailRef = useRef<TextInput | null>(null);
  const phoneNumberRef = useRef<TextInput | null>(null);
  const passwordRef = useRef<TextInput | null>(null);

  const onSubmit = (values: Record<string, any>) => {
    submit({
      ...editingUser,
      ...values,
      roleNames,
    });
  };

  const passwordValidation = Yup.lazy(() => {
    if (editingUser.id) {
      return Yup.string();
    }
    return Yup.string().required(i18n.t('AbpAccount::ThisFieldIsRequired'));
  });

  const formik = useFormik({
    enableReinitialize: true,
    validationSchema: Yup.object().shape({
      ...validations,
      password: passwordValidation,
    }),
    initialValues: {
      lockoutEnabled: false,
      ...editingUser,
    },
    onSubmit,
  });
  const {
    values,
    errors,
    touched,
    handleChange,
    handleBlur,
    handleSubmit,
    isValid,
  } = formik;

  return (
    <>
      <KeyboardAvoidingView
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      >
        <Box w="$full" px="$4" pb={96}>
          <HStack mx="auto" space="sm" my="$2" justifyContent="center">
            <Button
              size="sm"
              variant={selectedTab === 0 ? 'solid' : 'outline'}
              onPress={() => setSelectedTab(0)}
            >
              <ButtonText>{i18n.t('AbpIdentity::UserInformations')}</ButtonText>
            </Button>
            <Button
              size="sm"
              variant={selectedTab === 1 ? 'solid' : 'outline'}
              onPress={() => setSelectedTab(1)}
            >
              <ButtonText>{i18n.t('AbpIdentity::Roles')}</ButtonText>
            </Button>
          </HStack>

          {selectedTab === 0 ? (
            <>
              <FormControl isRequired my="$2" isInvalid={!!errors.userName}>
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
                      onChangeText={handleChange('userName')}
                      onBlur={handleBlur('userName')}
                      value={values.userName}
                      autoCapitalize="none"
                      color={inputColors.textColor}
                      placeholderTextColor={inputColors.placeholderColor}
                      selectionColor={inputColors.textColor}
                    />
                  </Input>
                  <ValidationMessage>{errors.userName}</ValidationMessage>
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
                    isInvalid={!!(errors.name && touched.name)}
                    borderWidth={1}
                    borderRadius="$lg"
                    bg={inputColors.bg}
                    borderColor={inputColors.borderColor}
                  >
                    <InputField
                      ref={nameRef}
                      onSubmitEditing={() => surnameRef.current?.focus()}
                      returnKeyType="next"
                      onChangeText={handleChange('name')}
                      onBlur={handleBlur('name')}
                      value={values.name}
                      placeholder={i18n.t('AbpIdentity::Name')}
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
                    isInvalid={!!(errors.surname && touched.surname)}
                    borderWidth={1}
                    borderRadius="$lg"
                    bg={inputColors.bg}
                    borderColor={inputColors.borderColor}
                  >
                    <InputField
                      ref={surnameRef}
                      onSubmitEditing={() => emailRef.current?.focus()}
                      returnKeyType="next"
                      onChangeText={handleChange('surname')}
                      onBlur={handleBlur('surname')}
                      value={values.surname}
                      placeholder={i18n.t('AbpIdentity::Surname')}
                      color={inputColors.textColor}
                      placeholderTextColor={inputColors.placeholderColor}
                      selectionColor={inputColors.textColor}
                    />
                  </Input>
                </VStack>
              </FormControl>

              <FormControl isRequired my="$2" isInvalid={!!errors.email}>
                <VStack mx="$4" space="xs">
                  <FormControlLabel>
                    <FormControlLabelText>
                      {i18n.t('AbpIdentity::EmailAddress')}
                    </FormControlLabelText>
                  </FormControlLabel>
                  <Input
                    isInvalid={!!(errors.email && touched.email)}
                    borderWidth={1}
                    borderRadius="$lg"
                    bg={inputColors.bg}
                    borderColor={inputColors.borderColor}
                  >
                    <InputField
                      ref={emailRef}
                      onSubmitEditing={() => phoneNumberRef.current?.focus()}
                      returnKeyType="next"
                      onChangeText={handleChange('email')}
                      onBlur={handleBlur('email')}
                      value={values.email}
                      autoCapitalize="none"
                      placeholder={i18n.t('AbpIdentity::EmailAddress')}
                      color={inputColors.textColor}
                      placeholderTextColor={inputColors.placeholderColor}
                      selectionColor={inputColors.textColor}
                    />
                  </Input>
                  <ValidationMessage>{errors.email}</ValidationMessage>
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
                    isInvalid={!!(errors.phoneNumber && touched.phoneNumber)}
                    borderWidth={1}
                    borderColor={inputColors.borderColor}
                    borderRadius="$lg"
                    bg={inputColors.bg}
                  >
                    <InputField
                      ref={phoneNumberRef}
                      onSubmitEditing={() => passwordRef?.current?.focus()}
                      returnKeyType={!editingUser.id ? 'next' : 'default'}
                      onChangeText={handleChange('phoneNumber')}
                      onBlur={handleBlur('phoneNumber')}
                      value={values.phoneNumber}
                      placeholder={i18n.t('AbpIdentity::PhoneNumber')}
                      color={inputColors.textColor}
                      placeholderTextColor={inputColors.placeholderColor}
                      selectionColor={inputColors.textColor}
                    />
                  </Input>
                </VStack>
              </FormControl>

              {!editingUser.id ? (
                <FormControl isRequired my="$2" isInvalid={!!errors.password}>
                  <VStack mx="$4" space="xs">
                    <FormControlLabel>
                      <FormControlLabelText>
                        {i18n.t('AbpIdentity::Password')}
                      </FormControlLabelText>
                    </FormControlLabel>
                    <Input
                      isInvalid={!!(errors.password && touched.password)}
                      borderWidth={1}
                      borderColor={inputColors.borderColor}
                      borderRadius="$lg"
                      bg={inputColors.bg}
                    >
                      <InputField
                        ref={passwordRef}
                        type={showPassword ? 'text' : 'password'}
                        onChangeText={handleChange('password')}
                        onBlur={handleBlur('password')}
                        value={values.password}
                        autoCapitalize="none"
                        placeholder={i18n.t('AbpIdentity::Password')}
                        color={inputColors.textColor}
                        placeholderTextColor={inputColors.placeholderColor}
                        selectionColor={inputColors.textColor}
                      />
                      <InputSlot
                        pr="$3"
                        onPress={() => setShowPassword(!showPassword)}
                      >
                        <Icon
                          as={Ionicons}
                          size="xl"
                          name={
                            showPassword ? 'eye-off-outline' : 'eye-outline'
                          }
                          color={inputColors.iconColor}
                        />
                      </InputSlot>
                    </Input>
                    <ValidationMessage>{errors.password}</ValidationMessage>
                  </VStack>
                </FormControl>
              ) : null}

              <FormControl my="$2">
                <VStack mx="$4" space="xs">
                  <Checkbox
                    value="lockout"
                    isChecked={values.lockoutEnabled}
                    onChange={isChecked =>
                      formik.setFieldValue('lockoutEnabled', isChecked)
                    }
                  >
                    <CheckboxIndicator>
                      <CheckboxIcon as={CheckIcon} />
                    </CheckboxIndicator>
                    <CheckboxLabel>
                      {' '}
                      {i18n.t('AbpIdentity::DisplayName:LockoutEnabled')}
                    </CheckboxLabel>
                  </Checkbox>
                </VStack>
              </FormControl>
            </>
          ) : (
            <UserRoles {...{ editingUser, onChangeRoles }} />
          )}
        </Box>
      </KeyboardAvoidingView>
      <FormButtonBar submit={handleSubmit} isSubmitDisabled={!isValid} />
    </>
  );
}

export default CreateUpdateUserForm;
