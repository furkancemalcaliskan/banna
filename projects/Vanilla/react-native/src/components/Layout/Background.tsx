import React from 'react';
import { Box } from '@gluestack-ui/themed';
import { useSelector } from 'react-redux';
import { useColorScheme } from 'react-native';
import { getColorToken, normalizeColorMode } from '../../theme';
import { createColorModeSelector } from '../../store/selectors/PersistentStorageSelectors';

type BackgroundProps = {
    children?: React.ReactNode;
};

const Background: React.FC<BackgroundProps> = ({ children }) => {
    const colorMode = useSelector(createColorModeSelector());
    const systemScheme = useColorScheme();
    const activeMode = normalizeColorMode(colorMode, systemScheme);
    const { value } = getColorToken(activeMode, 'backgroundLight', 'backgroundDark');

    return (
        <Box
            flex={1}
            style={{ backgroundColor: value }}
        >
            {children}
        </Box>
    );
};

export default Background;
