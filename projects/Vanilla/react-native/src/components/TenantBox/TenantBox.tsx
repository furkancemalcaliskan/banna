import i18n from '../../utils/i18n';
import {
  Box,
  Button,
  ButtonText,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  Input,
  InputField,
  Text,
  VStack,
} from '@gluestack-ui/themed';
import React, { forwardRef, useState } from 'react';
import { Alert, StyleSheet, View, StyleProp, ViewStyle } from 'react-native';
import { getTenant } from '../../api/AccountAPI';
import PersistentStorageActions from '../../store/actions/PersistentStorageActions';
import { createTenantSelector } from '../../store/selectors/PersistentStorageSelectors';
import { connectToRedux } from '../../utils/ReduxConnect';

type Tenant = {
  name?: string;
  [key: string]: any;
};

type TenantBoxProps = {
  tenant: Tenant;
  setTenant: (next: Tenant) => void;
  showTenantSelection: boolean;
  toggleTenantSelection: () => void;
  style?: StyleProp<ViewStyle>;
};

function TenantBox({
  tenant = {},
  setTenant,
  showTenantSelection,
  toggleTenantSelection,
}: TenantBoxProps) {
  const [tenantName, setTenantName] = useState(tenant.name);

  const findTenant = () => {
    if (!tenantName) {
      setTenant({});
      toggleTenantSelection();
      return;
    }

    getTenant(tenantName).then(({ success, ...data }) => {
      if (!success) {
        Alert.alert(
          i18n.t('AbpUi::Error'),
          i18n.t('AbpUiMultiTenancy::GivenTenantIsNotAvailable', {
            0: tenantName,
          }),
          [{ text: i18n.t('AbpUi::Ok') }]
        );
        return;
      }
      setTenant(data);
      toggleTenantSelection();
    });
  };

  return (
    <>
      <Box
        mb="$5"
        px="$4"
        w="$full"
        flexDirection="row"
        justifyContent="space-between"
        alignItems="center"
      >
        <Box flex={1}>
          <Text
            fontSize="$xs"
            fontWeight="$semibold"
            textTransform="uppercase"
            color="$textLight900"
            _dark={{ color: '$textDark' }}
            mb="$1"
          >
            {i18n.t('AbpUiMultiTenancy::Tenant')}
          </Text>
          <Text
            color="$textLight700"
            _dark={{ color: '$textDarkMuted' }}
          >
            {tenant.name
              ? tenant.name
              : i18n.t('AbpUiMultiTenancy::NotSelected')}
          </Text>
        </Box>
        {!showTenantSelection ? (
          <Button
            size="sm"
            onPress={() => toggleTenantSelection()}
          >
            <ButtonText>{i18n.t('AbpUiMultiTenancy::Switch')}</ButtonText>
          </Button>
        ) : null}
      </Box>
      {showTenantSelection ? (
        <Box px="$3" w="$full">
          <FormControl my="$2">
            <VStack space="xs">
              <FormControlLabel>
                <FormControlLabelText>
                  {i18n.t('AbpUiMultiTenancy::Name')}
                </FormControlLabelText>
              </FormControlLabel>
              <Input>
                <InputField
                  autoCapitalize="none"
                  value={tenantName}
                  onChangeText={setTenantName}
                />
              </Input>
            </VStack>
          </FormControl>
          <Text color="$textLightMuted" _dark={{ color: '$textDarkMuted' }} fontSize="$sm">
            {i18n.t('AbpUiMultiTenancy::SwitchTenantHint')}
          </Text>
          <View
            style={{ flexDirection: 'row', justifyContent: 'space-between' }}
          >
            <Button
              style={styles.button}
              onPress={() => toggleTenantSelection()}
              variant="outline"
            >
              <ButtonText>{i18n.t('AbpAccount::Cancel')}</ButtonText>
            </Button>
            <Button style={styles.button} onPress={() => findTenant()}>
              <ButtonText>{i18n.t('AbpAccount::Save')}</ButtonText>
            </Button>
          </View>
        </Box>
      ) : null}
    </>
  );
}

const styles = StyleSheet.create({
  button: { marginTop: 20, width: '49%' },

  tenant: {},
  title: {
    marginRight: 10,
    fontSize: 13,
    fontWeight: '600',
    textTransform: 'uppercase',
  },
  hint: { textAlign: 'left' },
});

const Forwarded = forwardRef((props, ref) => (
  <TenantBox {...props} forwardedRef={ref} />
));

export default connectToRedux({
  component: Forwarded,
  dispatchProps: {
    setTenant: PersistentStorageActions.setTenant,
  },
  stateProps: (state) => ({
    tenant: createTenantSelector()(state),
  }),
});
