import { Ionicons } from '@expo/vector-icons';
import { Icon, MenuIcon } from '@gluestack-ui/themed';
import React from 'react';
import { Platform } from 'react-native';
import { useNavIconColor } from '../../theme';

export default function HamburgerIcon({ navigation, ...iconProps }) {
  const iconColor = useNavIconColor();
  // Using MenuIcon from Gluestack or Generic Icon with as={Ionicons}
  // Gluestack default MenuIcon is good for theme consistency.
  return (
    <Icon
      as={MenuIcon}
      size="xl"
      ml="$3"
      mr="$2"
      color={iconProps.color || iconColor}
      onPress={() => navigation.openDrawer()}
      {...iconProps}
    />
  );
}
