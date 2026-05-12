import { useConfigStore } from '../store/config';
import styles from './UssrBanner.module.css';

type BannerId = 12 | 16 | 17 | 18 | 19 | 20 | 23 | 24 | 25 | 26 | 27 | 29;

interface UssrBannerProps {
  n: BannerId;
  /**
   * 'shrink' (default) — banner shrinks when container is narrow, centered at natural size.
   * 'fill' — banner fills 100% of the container width (use inside overflow:hidden cards).
   */
  variant?: 'shrink' | 'fill';
}

export function UssrBanner({ n, variant = 'shrink' }: UssrBannerProps) {
  const theme = useConfigStore((s) => s.theme);
  if (theme !== 'ussr') return null;

  return (
    <img
      src={`/banners/${n}.png`}
      alt=""
      role="presentation"
      className={variant === 'fill' ? styles.bannerFill : styles.banner}
    />
  );
}
