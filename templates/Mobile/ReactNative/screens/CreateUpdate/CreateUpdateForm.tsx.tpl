import * as Yup from 'yup';
import { useRef, useState } from 'react';
import { Platform, KeyboardAvoidingView, RefreshControl, StyleSheet, ScrollView, TextInput } from 'react-native';
import { useHeaderHeight } from '@react-navigation/elements';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useFormik } from 'formik';
import i18n from '../../../utils/i18n';
import {
  Box,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  Input,
  InputField,
  VStack,
__mobile_form_extra_imports__
} from '@gluestack-ui/themed';
__mobile_form_external_imports__
import { useInputColors } from '../../../theme';

import { FormButtons } from '../../../components/FormButtons';
import ValidationMessage from '../../../components/ValidationMessage/ValidationMessage';

const validations = {
__mobile_form_validations__
};

type CreateUpdate__entity_name__FormProps = {
  submit: (data: Record<string, any>) => void;
  __entity_name_lower__?: Record<string, any> | null;
  refreshing?: boolean;
  onRefresh?: () => void;
__mobile_form_props__
  [key: string]: any;
};

function CreateUpdate__entity_name__Form({
  submit,
  __entity_name_lower__ = null,
  refreshing = false,
  onRefresh = () => {},
__mobile_form_props__
}: CreateUpdate__entity_name__FormProps) {
  const headerHeight = useHeaderHeight();
  const insets = useSafeAreaInsets();
  const topInset = headerHeight > 0 ? 0 : insets.top || 0;
  const inputColors = useInputColors();
  const inputBg = inputColors.bg;
  const inputBgDark = inputColors.bg;
  const inputBorder = inputColors.borderColor;
  const inputBorderDark = inputColors.borderColor;

__mobile_form_state__

__mobile_form_refs__

  const onSubmit = (values: Record<string, any>) => {
    if (!form.isValid) {
      return;
    }

    submit({ ...values });
  };

  const form = useFormik<Record<string, any>>({
    enableReinitialize: true,
    validateOnBlur: true,
    validationSchema: Yup.object().shape({
      ...validations,
    }),
    initialValues: {
__mobile_form_initial_values__
    },
    onSubmit,
  });

  const isInvalidControl = (controlName = null) => {
    if (!controlName) {
      return;
    }

    return (
      ((!!form.touched[controlName] && form.submitCount > 0) || form.submitCount > 0) &&
      !!form.errors[controlName]
    );
  };

  return (
    <Box flex={1} bg="transparent" _dark={{ bg: 'transparent' }}>
__mobile_form_modals__
      <KeyboardAvoidingView behavior={Platform.OS === 'ios' ? 'padding' : 'height'}>
        <ScrollView
          keyboardShouldPersistTaps="handled"
          contentContainerStyle={[styles.content, { paddingTop: headerHeight || topInset }]}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        >
          <VStack space="md" px="$4" py="$2">
__mobile_form_inputs__
          </VStack>
        </ScrollView>
      </KeyboardAvoidingView>
      <Box
        position="absolute"
        left="$0"
        right="$0"
        bottom="$0"
        px="$4"
        py="$3"
        bg="transparent"
        borderTopWidth="$1"
        borderColor={inputBorder}
        _dark={{ bg: 'transparent', borderColor: inputBorderDark }}
      >
        <FormButtons submit={form.handleSubmit} isFixed={false} />
      </Box>
    </Box>
  );
}

const styles = StyleSheet.create({
  content: {
    paddingBottom: 96,
  },
__mobile_form_styles__
});

export default CreateUpdate__entity_name__Form;
