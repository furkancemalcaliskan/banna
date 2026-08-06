import { Box, HStack, Pressable, Text, Icon } from '@gluestack-ui/themed';
import { Ionicons } from '@expo/vector-icons';
import React, { useState } from 'react';
import { useSelector } from 'react-redux';
import { Alert } from 'react-native';
import { getTenants, removeTenant } from '../../api/TenantManagementAPI';
import DataList from '../../components/DataList/DataList';
import Card from '../../components/Layout/Card';
import ContextMenuPopover from '../../components/ContextMenuPopover/ContextMenuPopover';
import { LocalizationContext } from '../../contexts/LocalizationContext';
import { createAppConfigSelector } from '../../store/selectors/AppSelectors';
import { useSurfaceColors } from '../../theme';

type TenantsScreenProps = {
  navigation: any;
};

function TenantsScreen({ navigation }: TenantsScreenProps) {
  const { t } = React.useContext(LocalizationContext);
  const policies = useSelector(createAppConfigSelector())?.auth?.grantedPolicies;
  const hasContextActions =
    !!policies &&
    (policies['AbpTenantManagement.Tenants.Edit'] ||
      policies['AbpTenantManagement.Tenants.Delete']);
  const [refresh, setRefresh] = useState<number | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [activeMenuId, setActiveMenuId] = useState<string | number | null>(null);

  const onRefresh = () => {
    setRefreshing(true);
    setRefresh((prev) => (prev ?? 0) + 1);
  };

  const removeOnClick = (item: any) => {
    Alert.alert(
      t('AbpUi::AreYouSure'),
      t('AbpTenantManagement::TenantDeletionConfirmationMessage', { 0: item?.name || '' }),
      [
        {
          text: t('AbpUi::Cancel'),
          style: 'cancel',
        },
        {
          style: 'default',
          text: t('AbpUi::Ok'),
          onPress: () => {
            removeTenant(item.id).then(() => {
              setRefresh((prev) => (prev ?? 0) + 1);
            });
          },
        },
      ],
      { cancelable: true },
    );
  };
  const surface = useSurfaceColors();

  return (
    <Box flex={1}>
      <DataList
          navigation={navigation}
          fetchFn={getTenants}
          trigger={refresh}
          refreshing={refreshing}
          onRefresh={onRefresh}
          onRefreshComplete={() => setRefreshing(false)}
          render={({ item }) => (
            <Card mb="$3" mx="$1" p="$0" overflow="hidden">
              <Box p="$4">
                <HStack space="xs" alignItems="center">
                  <Text color="$secondary500" size="sm">{t('AbpTenantManagement::TenantName')}:</Text>
                  <Text fontWeight="$bold" size="md">
                    {item.name}
                  </Text>
                </HStack>
              </Box>
              {hasContextActions && (
                <ContextMenuPopover
                  isOpen={activeMenuId === item.id}
                  onOpen={() => setActiveMenuId(item.id)}
                  onClose={() => setActiveMenuId(null)}
                  actions={[
                    policies?.['AbpTenantManagement.Tenants.Edit']
                      ? {
                          label: t('AbpUi::Edit'),
                          iconName: 'create-outline',
                          onPress: () => navigateToCreateUpdateTenantScreen(navigation, item),
                        }
                      : null,
                    policies?.['AbpTenantManagement.Tenants.Delete']
                      ? {
                          label: t('AbpUi::Delete'),
                          isDestructive: true,
                          iconName: 'trash-outline',
                          onPress: () => removeOnClick(item),
                        }
                      : null,
                  ].filter(Boolean)}
                  triggerProps={{
                    position: 'absolute',
                    top: '$2',
                    right: '$2',
                    p: '$2',
                    hitSlop: 8,
                  }}
                />
              )}
            </Card>
          )}
        />
      <Pressable
        position="absolute"
        bottom="$6"
        right="$6"
        bg="$primary500"
        _dark={{ bg: '$primary400' }}
        borderRadius="$full"
        p="$4"
        shadowColor={surface.shadowColor}
        shadowOpacity={0.25}
        shadowRadius={12}
        elevation={8}
        onPress={() => navigateToCreateUpdateTenantScreen(navigation)}
        accessibilityLabel="Add tenant"
      >
        <Icon as={Ionicons} name="add" size="xl" color="$white" />
      </Pressable>
    </Box>
  );
}

const navigateToCreateUpdateTenantScreen = (navigation: any, tenant: any = {}) => {
  navigation.navigate('CreateUpdateTenant', {
    tenantId: tenant.id,
  });
};

export default TenantsScreen;
