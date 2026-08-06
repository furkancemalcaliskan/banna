import { useFormik } from 'formik';
import i18n from '../../utils/i18n';
import {
  Box,
  Button,
  ButtonText,
  Center,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  FormControlHelper,
  FormControlHelperText,
  FormControlError,
  FormControlErrorText,
  FormControlErrorIcon,
  Image,
  Input,
  InputField,
  VStack,
  AlertCircleIcon,
  Checkbox,
  CheckboxIndicator,
  CheckboxIcon,
  CheckboxLabel,
  CheckIcon,
  InputSlot,
  Icon,
} from '@gluestack-ui/themed';
import { Ionicons } from '@expo/vector-icons';
import React, { useRef, useState } from 'react';
import { KeyboardAvoidingView, Platform, ScrollView, View } from 'react-native';
import { object, string } from 'yup';
import { login } from '../../api/AccountAPI';
import TenantBox from '../../components/TenantBox/TenantBox';
import Card from '../../components/Layout/Card';
import ValidationMessage from '../../components/ValidationMessage/ValidationMessage';
import AppActions from '../../store/actions/AppActions';
import LoadingActions from '../../store/actions/LoadingActions';
import PersistentStorageActions from '../../store/actions/PersistentStorageActions';
import { connectToRedux } from '../../utils/ReduxConnect';
import { useInputColors } from '../../theme';

type LoginScreenProps = {
  startLoading: (payload: any) => void;
  stopLoading: (payload: any) => void;
  setToken: (token: any) => void;
  fetchAppConfig: (payload: any) => void;
};

const ValidationSchema = object().shape({
  username: string().required(i18n.t('AbpAccount::ThisFieldIsRequired')),
  password: string().required(i18n.t('AbpAccount::ThisFieldIsRequired')),
});

const LoginScreen: React.FC<LoginScreenProps> = ({ startLoading, stopLoading, setToken, fetchAppConfig }) => {
  const [showTenantSelection, setShowTenantSelection] = useState(false);
  const [rememberMe, setRememberMe] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const passwordRef = useRef<any>(null);
  const inputColors = useInputColors();

  const toggleTenantSelection = () => {
    setShowTenantSelection(!showTenantSelection);
  };

  const submit = ({ username, password }: { username: string; password: string }) => {
    startLoading({ key: 'login' });
    login({ username, password })
      .then(data => {
        let expireTime;
        if (rememberMe) {
          // Set expiration to 10 years from now if Remember Me is checked
          expireTime = new Date().valueOf() + 3650 * 24 * 60 * 60 * 1000;
        } else {
          // Otherwise use the token's expiration
          expireTime = new Date().valueOf() + data.expires_in * 1000;
        }

        setToken({
          ...data,
          expire_time: expireTime,
          scope: undefined,
        });
      })
      .then(
        () =>
          new Promise(resolve =>
            fetchAppConfig({
              showLoading: false,
              callback: () => resolve(true),
            }),
          ),
      )
      .finally(() => stopLoading({ key: 'login' }));
  };

  const formik = useFormik({
    validationSchema: ValidationSchema,
    initialValues: { username: '', password: '' },
    onSubmit: submit,
  });

  return (
    <KeyboardAvoidingView
      style={{ flex: 1 }}
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      keyboardVerticalOffset={Platform.OS === 'ios' ? 0 : 20}
    >
      <ScrollView
        contentContainerStyle={{ flexGrow: 1 }}
        keyboardShouldPersistTaps="handled"
      >
        <Center flex={1} px="$4" py="$4">
          <Card w="$full" maxWidth={400} p="$6">
            <Box w="$full" mb="$8" alignItems="center">
              <Image
                alt="MyProjectName Logo"
                source={require('../../../assets/logo.png')}
                style={{ width: 150, height: 150, resizeMode: 'contain' }}
              />
            </Box>

            <TenantBox
              showTenantSelection={showTenantSelection}
              toggleTenantSelection={toggleTenantSelection}
            />

            <Box w="$full" display={showTenantSelection ? 'none' : 'flex'}>
              <FormControl isRequired my="$2" isInvalid={!!formik.errors.username}>
                <VStack space="xs">
                  <FormControlLabel>
                    <FormControlLabelText>
                      {i18n.t('AbpAccount::UserNameOrEmailAddress')}
                    </FormControlLabelText>
                  </FormControlLabel>
                  <Input
                    size="lg"
                    bg={inputColors.bg}
                    borderWidth={1}
                    borderColor={inputColors.borderColor}
                    borderRadius="$lg"
                  >
                    <InputField
                      onChangeText={formik.handleChange('username')}
                      onBlur={formik.handleBlur('username')}
                      value={formik.values.username}
                      returnKeyType="next"
                      autoCapitalize="none"
                      onSubmitEditing={() => passwordRef?.current?.focus()}
                      color={inputColors.textColor}
                      placeholderTextColor={inputColors.placeholderColor}
                      selectionColor={inputColors.textColor}
                    />
                  </Input>
                  <FormControlError>
                    <FormControlErrorIcon as={AlertCircleIcon} />
                    <FormControlErrorText>
                      {formik.errors.username}
                    </FormControlErrorText>
                  </FormControlError>
                </VStack>
              </FormControl>

              <FormControl isRequired my="$2" isInvalid={!!formik.errors.password}>
                <VStack space="xs">
                  <FormControlLabel>
                    <FormControlLabelText>
                      {i18n.t('AbpAccount::Password')}
                    </FormControlLabelText>
                  </FormControlLabel>
                  <Input
                    size="lg"
                    bg={inputColors.bg}
                    borderWidth={1}
                    borderColor={inputColors.borderColor}
                    borderRadius="$lg"
                  >
                    <InputField
                      type={showPassword ? 'text' : 'password'}
                      onChangeText={formik.handleChange('password')}
                      onBlur={formik.handleBlur('password')}
                      value={formik.values.password}
                      ref={passwordRef}
                      autoCapitalize="none"
                      returnKeyType="done"
                      onSubmitEditing={formik.handleSubmit}
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
                        name={showPassword ? 'eye-off-outline' : 'eye-outline'}
                        size="xl"
                        color={inputColors.iconColor}
                      />
                    </InputSlot>
                  </Input>
                  <FormControlError>
                    <FormControlErrorIcon as={AlertCircleIcon} />
                    <FormControlErrorText>
                      {formik.errors.password}
                    </FormControlErrorText>
                  </FormControlError>
                </VStack>
              </FormControl>

              <Box my="$4">
                <Checkbox
                  size="md"
                  isInvalid={false}
                  isDisabled={false}
                  value="rememberMe"
                  isChecked={rememberMe}
                  onChange={setRememberMe}
                >
                  <CheckboxIndicator>
                    <CheckboxIcon as={CheckIcon} />
                  </CheckboxIndicator>
                  <CheckboxLabel> {i18n.t('AbpAccount::RememberMe')}</CheckboxLabel>
                </Checkbox>
              </Box>

              <View style={{ marginTop: 20, alignItems: 'center' }}>
                <Button
                  onPress={formik.handleSubmit}
                  width="$full"
                  size="lg"
                  boxShadow="$glow"
                >
                  <ButtonText>{i18n.t('AbpAccount::Login')}</ButtonText>
                </Button>
              </View>
            </Box>
          </Card>
        </Center>
      </ScrollView>
    </KeyboardAvoidingView>
  );
};

export default connectToRedux({
  component: LoginScreen,
  dispatchProps: {
    startLoading: LoadingActions.start,
    stopLoading: LoadingActions.stop,
    fetchAppConfig: AppActions.fetchAppConfigAsync,
    setToken: PersistentStorageActions.setToken,
  },
});
