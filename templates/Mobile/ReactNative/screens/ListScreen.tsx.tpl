import { useState } from 'react';
import { useSelector } from 'react-redux';
import { Alert, StyleSheet } from 'react-native';
import React from 'react';
import { Box, Icon, Pressable, Text, VStack } from '@gluestack-ui/themed';
import { Ionicons } from '@expo/vector-icons';

import { getList, remove } from '../../api/__entity_name__API';
import DataList from '../../components/DataList/DataList';
import Card from '../../components/Layout/Card';
import ContextMenuPopover from '../../components/ContextMenuPopover/ContextMenuPopover';
import { createAppConfigSelector } from '../../store/selectors/AppSelectors';
import { LocalizationContext } from '../../contexts/LocalizationContext';
import { useSurfaceColors } from '../../theme';

type __entity_plural__ScreenProps = {
  navigation: any;
};

function __entity_plural__Screen({ navigation }: __entity_plural__ScreenProps) {
  const { t } = React.useContext(LocalizationContext);
  const currentUser = useSelector(createAppConfigSelector())?.currentUser;
  const policies = useSelector(createAppConfigSelector())?.auth?.grantedPolicies;
  const hasContextActions =
    !!policies &&
    (policies['__domain_short__.__entity_plural__.Edit'] ||
      policies['__domain_short__.__entity_plural__.Delete']);

  const [refresh, setRefresh] = useState<number | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [activeMenuId, setActiveMenuId] = useState<string | number | null>(null);

  const onRefresh = () => {
    setRefreshing(true);
    setRefresh((prev) => (prev ?? 0) + 1);
  };

  const removeOnClick = (item: any) => {
    Alert.alert('Warning', t('__domain_short__::AreYouSureToDelete'), [
      {
        text: t('AbpUi::Cancel'),
        style: 'cancel',
      },
      {
        style: 'default',
        text: t('AbpUi::Ok'),
        onPress: () => {
          remove(item.id).then(() => {
            setRefresh((prev) => (prev ?? 0) + 1);
          });
        },
      },
    ]);
  };

  const edit = (item: any) => {
    navigation.navigate('CreateUpdate__entity_name__', { __entity_name_lower__Id: item.id });
  };
  const surface = useSurfaceColors();

  return (
    <Box flex={1}>
      {currentUser?.isAuthenticated && (
        <DataList
          navigation={navigation}
          fetchFn={getList}
          refreshing={refreshing}
          onRefresh={onRefresh}
          onRefreshComplete={() => setRefreshing(false)}
          trigger={refresh}
          render={({ item }) => {
            const entityItem = item?.__entity_name_lower__ || item;
            const actions = [];

            if (policies?.['__domain_short__.__entity_plural__.Edit']) {
              actions.push({
                label: t('AbpUi::Edit'),
                iconName: 'create-outline',
                onPress: () => edit(entityItem),
              });
            }

            if (policies?.['__domain_short__.__entity_plural__.Delete']) {
              actions.push({
                label: t('AbpUi::Delete'),
                isDestructive: true,
                iconName: 'trash-outline',
                onPress: () => removeOnClick(entityItem),
              });
            }

            return (
              <Card mb="$3" mx="$1" p="$0" overflow="hidden">
              <Box p="$3">
                <VStack space="sm">
                  <Text fontWeight="$bold">{__mobile_list_title__}</Text>
__mobile_list_fields__
                </VStack>
              </Box>
                {hasContextActions && (
                  <ContextMenuPopover
                    isOpen={activeMenuId === entityItem.id}
                    onOpen={() => setActiveMenuId(entityItem.id)}
                    onClose={() => setActiveMenuId(null)}
                    actions={actions}
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
            );
          }}
        />
      )}

      {currentUser?.isAuthenticated && !!policies['__domain_short__.__entity_plural__.Create'] && (
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
          onPress={() => navigation.navigate('CreateUpdate__entity_name__')}
          accessibilityLabel="Add __entity_name_lower__"
        >
          <Icon as={Ionicons} name="add" size="xl" color="$white" />
        </Pressable>
      )}
    </Box>
  );
}

const styles = StyleSheet.create({
  container: {
    flexGrow: 1,
  },
  fabStyle: {
    bottom: 16,
    right: 16,
    position: 'absolute',
  },
});

export default __entity_plural__Screen;
