import type { Preview, Decorator } from '@storybook/react';
import '../src/styles/theme.css';

const withTheme: Decorator = (Story, context) => {
  const theme = context.globals['theme'] as string | undefined;
  if (theme === 'ussr') {
    document.documentElement.classList.add('theme-ussr');
    // USSR: content components live on --card-bg (cream), not --detail-bg (near-black)
    document.body.style.background = 'var(--card-bg)';
    document.body.style.padding = '24px';
  } else {
    document.documentElement.classList.remove('theme-ussr');
    document.body.style.background = '';
    document.body.style.padding = '';
  }
  return Story();
};

const preview: Preview = {
  decorators: [withTheme],
  globalTypes: {
    theme: {
      name: 'Theme',
      description: 'Global theme switch',
      defaultValue: 'catppuccin',
      toolbar: {
        icon: 'paintbrush',
        items: [
          { value: 'catppuccin', title: 'Catppuccin Frappé' },
          { value: 'ussr', title: 'USSR' },
        ],
        showName: true,
        dynamicTitle: true,
      },
    },
  },
  parameters: {
    backgrounds: { disable: true },
    layout: 'centered',
  },
};

export default preview;
