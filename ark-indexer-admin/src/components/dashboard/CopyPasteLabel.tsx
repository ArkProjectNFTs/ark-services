import useClipboard from "react-use-clipboard";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";

interface CopyPasteLabelProps {
  label: string;
}

const CopyPasteLabel = (props: CopyPasteLabelProps) => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [isCopied, setCopied] = useClipboard(props.label);

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger>
          <div onClick={setCopied}>{props.label}</div>
        </TooltipTrigger>
        <TooltipContent>
          <p>Copy to clipboard</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
};

export default CopyPasteLabel;
