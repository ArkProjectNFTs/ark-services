import Image from "next/image";
import { useTheme } from "next-themes";

import { useIsSSR } from "../hooks/useIsSSR";

export default function DarkModeButton() {
  const isSSR = useIsSSR();

  const { setTheme, theme } = useTheme();

  function toggleTheme() {
    if (theme === "light") {
      setTheme("dark");
      return;
    }
    setTheme("light");
  }

  if (isSSR) {
    return null;
  }

  return (
    // TODO @YohanTz: transition
    <button onClick={toggleTheme}>
      <Image
        className="hidden dark:block"
        src="/icons/darkMode.svg"
        alt={"Dark mode icon"}
        height={32}
        width={32}
      />
      <Image
        className="block dark:hidden"
        src="/icons/lightMode.svg"
        alt="Light mode icon"
        height={32}
        width={32}
      />
    </button>
  );
}
