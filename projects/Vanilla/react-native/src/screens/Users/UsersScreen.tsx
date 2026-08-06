import { Box, HStack, Pressable, Text, VStack, Icon } from '@gluestack-ui/themed';
import { Ionicons } from '@expo/vector-icons';
import React, { useState } from 'react';
import { useSelector } from 'react-redux';
import { Alert } from 'react-native';
import { getUsers, removeUser } from '../../api/IdentityAPI';
import DataList from '../../components/DataList/DataList';
import Card from '../../components/Layout/Card';
import ContextMenuPopover from '../../components/ContextMenuPopover/ContextMenuPopover';
import { LocalizationContext } from '../../contexts/LocalizationContext';
import { createAppConfigSelector } from '../../store/selectors/AppSelectors';
import { useSurfaceColors } from '../../theme';

type UsersScreenProps = {
  navigation: any;
};

function UsersScreen({ navigation }: UsersScreenProps) {
  const { t } = React.useContext(LocalizationContext);
  const policies = useSelector(createAppConfigSelector())?.auth?.grantedPolicies;
  const hasContextActions =
    !!policies &&
    (policies['AbpIdentity.Users.Edit'] || policies['AbpIdentity.Users.Delete']);
  const [refresh, setRefresh] = useState<number | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [activeMenuId, setActiveMenuId] = useState<string | number | null>(null);

  const onRefresh = () => {
    setRefreshing(true);
    setRefresh((prev) => (prev ?? 0) + 1);
  };
  const surface = useSurfaceColors();

  const removeOnClick = (item: any) => {
    const name = item?.userName || item?.email || '';
    Alert.alert(
      t('AbpUi::AreYouSure'),
      t('AbpIdentity::UserDeletionConfirmationMessage', { 0: name }),
      [
        {
          text: t('AbpUi::Cancel'),
          style: 'cancel',
        },
        {
          style: 'default',
          text: t('AbpUi::Ok'),
          onPress: () => {
            removeUser(item.id).then(() => {
              setRefresh((prev) => (prev ?? 0) + 1);
            });
          },
        },
      ],
      { cancelable: true },
    );
  };

  return (
    <Box flex={1}>
      <DataList
        navigation={navigation}
        fetchFn={getUsers}
        trigger={refresh}
        refreshing={refreshing}
        onRefresh={onRefresh}
        onRefreshComplete={() => setRefreshing(false)}
        render={({ item }) => (
          <Card mb="$3" mx="$1" p="$0" overflow="hidden">
            <Box p="$3">
              <VStack space="sm">
                <HStack space="xs" alignItems="center">
                  <Text color="$secondary500" size="sm">{t('AbpIdentity::UserName')}:</Text>
                  <Text fontWeight="$bold">
                    {item.userName}
                  </Text>
                </HStack>
                <HStack space="xs" alignItems="center">
                  <Text color="$secondary500" size="sm">{t('AbpIdentity::EmailAddress')}:</Text>
                  <Text fontWeight="$medium">
                    {item.email}
                  </Text>
                </HStack>
                <HStack space="xs" alignItems="center">
                  <Text color="$secondary500" size="sm">{t('AbpIdentity::DisplayName:Name')}:</Text>
                  <Text>
                    {item.name}
                  </Text>
                </HStack>
                <HStack space="xs" alignItems="center">
                  <Text color="$secondary500" size="sm">{t('AbpIdentity::DisplayName:Surname')}:</Text>
                  <Text>
                    {item.surname}
                  </Text>
                </HStack>
              </VStack>
            </Box>
            {hasContextActions && (
              <ContextMenuPopover
                isOpen={activeMenuId === item.id}
                onOpen={() => setActiveMenuId(item.id)}
                onClose={() => setActiveMenuId(null)}
                actions={[
                  policies?.['AbpIdentity.Users.Edit']
                    ? {
                        label: t('AbpUi::Edit'),
                        iconName: 'create-outline',
                        onPress: () => navigateToCreateUpdateUserScreen(navigation, item),
                      }
                    : null,
                  policies?.['AbpIdentity.Users.Delete']
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
      onPress={() => navigateToCreateUpdateUserScreen(navigation)}
      accessibilityLabel="Add user"
      >
        <Icon as={Ionicons} name="add" size="xl" color="$white" />
      </Pressable>
    </Box>
  );
}

const navigateToCreateUpdateUserScreen = (navigation: any, user: any = {}) => {
  navigation.navigate('CreateUpdateUser', {
    userId: user.id,
  });
};

export default UsersScreen;
