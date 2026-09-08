import mark from '../assets/dynam-mark.svg';

/** Decorative mark; the surrounding heading supplies the accessible name. */
export function DynamMark({ size = 32 }: { size?: number }) {
  return <img src={mark} width={size} height={size} alt="" aria-hidden="true" />;
}
