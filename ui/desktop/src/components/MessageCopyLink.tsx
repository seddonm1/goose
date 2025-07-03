/* global ClipboardItem */

import React, { useState } from 'react';
import { Copy } from './icons';

interface MessageCopyLinkProps {
  text: string;
  contentRef: React.RefObject<HTMLElement>;
}

export default function MessageCopyLink({ text, contentRef }: MessageCopyLinkProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      if (contentRef?.current) {
        // Create a temporary container to handle HTML content
        const container = document.createElement('div');
        container.innerHTML = contentRef.current.innerHTML;

        // Clean up any copy buttons from the content
        const copyButtons = container.querySelectorAll('button');
        copyButtons.forEach((button) => button.remove());

        // Create the clipboard data
        const clipboardData = new ClipboardItem({
          'text/plain': new Blob([text], { type: 'text/plain' }),
          'text/html': new Blob([container.innerHTML], { type: 'text/html' }),
        });

        await navigator.clipboard.write([clipboardData]);
      } else {
        await navigator.clipboard.writeText(text);
      }

      setCopied(true);
      setTimeout(() => setCopied(false), 2000); // Reset after 2 seconds
    } catch (err) {
      console.error('Failed to copy text: ', err);
      // Fallback to plain text if HTML copy fails
      try {
        await navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (fallbackErr) {
        console.error('Failed to copy text (fallback): ', fallbackErr);
      }
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="flex items-center gap-1 text-xs text-textSubtle hover:cursor-pointer hover:text-textProminent transition-all duration-200 opacity-0 group-hover:opacity-100 -translate-y-4 group-hover:translate-y-0"
    >
      <Copy className="h-3 w-3" />
      <span>{copied ? 'Copied!' : 'Copy'}</span>
    </button>
  );
}
