import { useTranslation } from 'react-i18next';
import { Button, Dropdown } from 'antd';
import { GlobalOutlined } from '@ant-design/icons';
import type { MenuProps } from 'antd';

export const LanguageSwitcher = () => {
  const { i18n } = useTranslation();

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng);
    localStorage.setItem('language', lng);
  };

  const items: MenuProps['items'] = [
    {
      key: 'en',
      label: 'English',
      onClick: () => changeLanguage('en'),
    },
    {
      key: 'zh',
      label: '中文',
      onClick: () => changeLanguage('zh'),
    },
  ];

  const getCurrentLabel = () => {
    return i18n.language === 'zh' ? '中文' : 'EN';
  };

  return (
    <Dropdown menu={{ items }} placement="bottomRight">
      <Button
        type="text"
        icon={<GlobalOutlined />}
        className="text-white hover:bg-white/10"
      >
        {getCurrentLabel()}
      </Button>
    </Dropdown>
  );
};
