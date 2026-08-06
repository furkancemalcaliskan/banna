import React from 'react';
import { StyleSheet } from 'react-native';
import { Box } from '@gluestack-ui/themed';

export default function HeaderBackground() {
    return (
        <Box
            flex={1}
            bg="$cardBg"
            borderBottomWidth={1}
            borderBottomColor="$borderColor"
            sx={{
                _dark: {
                    bg: '$sidebarBgDark',
                    borderColor: '$borderColorDark',
                }
            }}
            style={StyleSheet.absoluteFill}
        />
    );
}
