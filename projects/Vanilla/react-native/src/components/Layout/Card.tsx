
import React from 'react';
import { Box, BoxProps } from '@gluestack-ui/themed';
import { StyleSheet, StyleProp, ViewStyle } from 'react-native';

type CardProps = BoxProps & {
    children?: React.ReactNode;
    style?: StyleProp<ViewStyle>;
};

const Card: React.FC<CardProps> = ({ children, style, p = '$5', ...props }) => {
    return (
        <Box
            bg="$cardBg"
            borderColor="$borderColor"
            borderWidth={1}
            borderRadius="$xl"
            p={p}
            {...props}
            sx={{
                _dark: {
                    bg: '$cardBgDark',
                    borderColor: '$borderColorDark',
                },
                shadowColor: '$soft',
                shadowOffset: { width: 0, height: 10 },
                shadowOpacity: 0.12,
                shadowRadius: 20,
                elevation: 5,
            }}
            style={style}
        >
            {children}
        </Box>
    );
};

export default Card;
