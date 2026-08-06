import { Ionicons } from '@expo/vector-icons';
import { useFormik } from 'formik';
import i18n from '../../utils/i18n';
import {
  Box,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  Icon,
  Input,
  InputField,
  InputSlot,
  VStack,
} from '@gluestack-ui/themed';
import React, { useRef, useState } from 'react';
import * as Yup from 'yup';
import { TextInput } from 'react-native';
import { FormButtonBar } from '../../components/FormButtons';
import ValidationMessage from '../../components/ValidationMessage/ValidationMessage';
import { useInputColors } from '../../theme';

const validations = {
  name: Yup.string().required(i18n.t('AbpAccount::ThisFieldIsRequired')),
};

type CreateUpdateTenantFormProps = {
  editingTenant?: Record<string, any>;
  submit: (data: Record<string, any>) => void;
};

function CreateUpdateTenantForm({
  editingTenant = {},
  submit,
}: CreateUpdateTenantFormProps) {
  const tenantNameRef = useRef<TextInput | null>(null);
  const adminEmailRef = useRef<TextInput | null>(null);
  const adminPasswordRef = useRef<TextInput | null>(null);

  const [showAdminPassword, setShowAdminPassword] = useState(false);
  const inputColors = useInputColors();

  const adminEmailAddressValidation = Yup.lazy(() =>
    Yup.string()
      .required(i18n.t('AbpAccount::ThisFieldIsRequired'))
      .email(i18n.t('AbpAccount::ThisFieldIsNotAValidEmailAddress')),
  );

  const adminPasswordValidation = Yup.lazy(() =>
    Yup.string().required(i18n.t('AbpAccount::ThisFieldIsRequired')),
  );

  const onSubmit = (values: Record<string, any>) => {
    submit({
      ...editingTenant,
      ...values,
    });
  };

  const formik = useFormik({
    enableReinitialize: true,
    validationSchema: Yup.object().shape({
      ...validations,
      ...(!editingTenant.id && {
        adminEmailAddress: adminEmailAddressValidation,
        adminPassword: adminPasswordValidation,
      }),
    }),
    initialValues: {
      lockoutEnabled: false,
      twoFactorEnabled: false,
      ...editingTenant,
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
      <Box w="$full" px="$3" pb={96}>
        <FormControl
          isRequired
          my="$2"
          isInvalid={!!errors.name && touched.name}
        >
          <VStack mx="$4" space="xs">
            <FormControlLabel>
              <FormControlLabelText>
                {i18n.t('AbpTenantManagement::TenantName')}
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
                ref={tenantNameRef}
                placeholder={i18n.t('AbpTenantManagement::TenantName')}
                onChangeText={handleChange('name')}
                onBlur={handleBlur('name')}
                value={values.name}
                autoCapitalize="none"
                returnKeyType="next"
                color={inputColors.textColor}
                placeholderTextColor={inputColors.placeholderColor}
                selectionColor={inputColors.textColor}
              />
            </Input>
            <ValidationMessage>{errors.name}</ValidationMessage>
          </VStack>
        </FormControl>

        {!editingTenant.id ? (
          <>
            <FormControl
              isRequired
              my="$2"
              isInvalid={
                !!errors.adminEmailAddress && touched.adminEmailAddress
              }
            >
              <VStack mx="$4" space="xs">
                <FormControlLabel>
                  <FormControlLabelText>
                    {i18n.t(
                      'AbpTenantManagement::DisplayName:AdminEmailAddress',
                    )}
                  </FormControlLabelText>
                </FormControlLabel>
                <Input
                  isInvalid={
                    !!(errors.adminEmailAddress && touched.adminEmailAddress)
                  }
                  borderWidth={1}
                  borderRadius="$lg"
                  bg={inputColors.bg}
                  borderColor={inputColors.borderColor}
                >
                  <InputField
                    ref={adminEmailRef}
                    placeholder={i18n.t(
                      'AbpTenantManagement::DisplayName:AdminEmailAddress',
                    )}
                    onChangeText={handleChange('adminEmailAddress')}
                    onBlur={handleBlur('adminEmailAddress')}
                    value={values.adminEmailAddress}
                    autoCapitalize="none"
                    onSubmitEditing={() => adminPasswordRef?.current?.focus()}
                    returnKeyType="next"
                    color={inputColors.textColor}
                    placeholderTextColor={inputColors.placeholderColor}
                    selectionColor={inputColors.textColor}
                  />
                </Input>
                <ValidationMessage>
                  {errors.adminEmailAddress}
                </ValidationMessage>
              </VStack>
            </FormControl>

            <FormControl
              isRequired
              my="$2"
              isInvalid={!!errors.adminPassword && touched.adminPassword}
            >
              <VStack mx="$4" space="xs">
                <FormControlLabel>
                  <FormControlLabelText>
                    {i18n.t('AbpTenantManagement::DisplayName:AdminPassword')}
                  </FormControlLabelText>
                </FormControlLabel>
                <Input
                  isInvalid={!!(errors.adminPassword && touched.adminPassword)}
                  borderWidth={1}
                  borderRadius="$lg"
                  bg={inputColors.bg}
                  borderColor={inputColors.borderColor}
                >
                  <InputField
                    ref={adminPasswordRef}
                    placeholder={i18n.t(
                      'AbpTenantManagement::DisplayName:AdminPassword',
                    )}
                    returnKeyType="done"
                    type={showAdminPassword ? 'text' : 'password'}
                    onChangeText={handleChange('adminPassword')}
                    onBlur={handleBlur('adminPassword')}
                    value={values.adminPassword}
                    autoCapitalize="none"
                    color={inputColors.textColor}
                    placeholderTextColor={inputColors.placeholderColor}
                    selectionColor={inputColors.textColor}
                  />
                  <InputSlot
                    pr="$3"
                    onPress={() => setShowAdminPassword(!showAdminPassword)}
                  >
                    <Icon
                      as={Ionicons}
                      size="xl"
                      name={
                        showAdminPassword ? 'eye-off-outline' : 'eye-outline'
                      }
                      color={inputColors.iconColor}
                    />
                  </InputSlot>
                </Input>
                <ValidationMessage>
                  {formik.errors.adminPassword}
                </ValidationMessage>
              </VStack>
            </FormControl>
          </>
        ) : null}
      </Box>
      <FormButtonBar submit={handleSubmit} isSubmitDisabled={!isValid} />
    </>
  );
}

export default CreateUpdateTenantForm;
