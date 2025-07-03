import React, { useMemo, useState, useEffect, useRef } from 'react';
import { Buffer } from 'buffer';
import Copy from '../icons/Copy';
import { Card } from './card';

interface RecipeConfig {
  instructions?: string;
  activities?: string[];
  [key: string]: unknown;
}

interface DeepLinkModalProps {
  recipeConfig: RecipeConfig;
  onClose: () => void;
}

// Function to generate a deep link from a bot config
export function generateDeepLink(recipeConfig: RecipeConfig): string {
  const configBase64 = Buffer.from(JSON.stringify(recipeConfig)).toString('base64');
  const urlSafe = encodeURIComponent(configBase64);
  return `goose://bot?config=${urlSafe}`;
}

export function DeepLinkModal({ recipeConfig: initialRecipeConfig, onClose }: DeepLinkModalProps) {
  // Create editable state for the bot config
  const [recipeConfig, setRecipeConfig] = useState(initialRecipeConfig);
  const [instructions, setInstructions] = useState(initialRecipeConfig.instructions || '');
  const [activities, setActivities] = useState<string[]>(initialRecipeConfig.activities || []);
  const [activityInput, setActivityInput] = useState('');

  // Generate the deep link using the current bot config
  const deepLink = useMemo(() => {
    const currentConfig = {
      ...recipeConfig,
      instructions,
      activities,
    };
    return generateDeepLink(currentConfig);
  }, [recipeConfig, instructions, activities]);

  // Handle Esc key press
  useEffect(() => {
    const handleEscKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    // Add event listener
    document.addEventListener('keydown', handleEscKey);

    // Clean up
    return () => {
      document.removeEventListener('keydown', handleEscKey);
    };
  }, [onClose]);

  // Update the bot config when instructions or activities change
  useEffect(() => {
    setRecipeConfig({
      ...recipeConfig,
      instructions,
      activities,
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [instructions, activities]);

  // Handle adding a new activity
  const handleAddActivity = () => {
    if (activityInput.trim()) {
      setActivities([...activities, activityInput.trim()]);
      setActivityInput('');
    }
  };

  // Handle removing an activity
  const handleRemoveActivity = (index: number) => {
    const newActivities = [...activities];
    newActivities.splice(index, 1);
    setActivities(newActivities);
  };

  // Reference for the modal content
  const modalRef = useRef<HTMLDivElement>(null);

  // Handle click outside the modal
  const handleBackdropClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (modalRef.current && !modalRef.current.contains(e.target as Node)) {
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black/20 dark:bg-white/20 backdrop-blur-sm transition-colors flex items-center justify-center p-4 z-50"
      onClick={handleBackdropClick}
    >
      <Card
        ref={modalRef}
        className="relative w-[700px] max-w-full bg-bgApp rounded-xl my-10 max-h-[90vh] flex flex-col shadow-lg"
      >
        <div className="p-8 overflow-y-auto" style={{ maxHeight: 'calc(90vh - 32px)' }}>
          <div className="flex flex-col">
            <h2 className="text-2xl font-bold mb-4 text-textStandard">Agent Created!</h2>
            <p className="mb-4 text-textStandard">
              Your agent has been created successfully. You can share or open it below:
            </p>

            {/* Sharable Goose Bot Section - Moved to top */}
            <div className="mb-6">
              <label className="block font-medium mb-1 text-textStandard">
                Sharable Goose Bot:
              </label>
              <div className="flex items-center">
                <input
                  type="text"
                  value={deepLink}
                  readOnly
                  className="flex-1 p-3 border border-borderSubtle rounded-l-md bg-transparent text-textStandard"
                />
                <button
                  onClick={() => {
                    navigator.clipboard.writeText(deepLink);
                    window.electron.logInfo('Deep link copied to clipboard');
                  }}
                  className="p-2 bg-blue-500 text-white rounded-r-md hover:bg-blue-600 flex items-center justify-center min-w-[100px]"
                >
                  <Copy className="w-5 h-5 mr-1" />
                  Copy
                </button>
              </div>
            </div>

            {/* Action Buttons - Moved to top */}
            <div className="flex mb-6">
              <button
                onClick={() => {
                  // Open the deep link with the current bot config
                  const currentConfig = {
                    id: 'deeplink-recipe',
                    name: 'DeepLink Recipe',
                    title: 'DeepLink Recipe',
                    description: 'Recipe from deep link',
                    ...recipeConfig,
                    instructions,
                    activities,
                  };
                  window.electron.createChatWindow(
                    undefined,
                    undefined,
                    undefined,
                    undefined,
                    currentConfig
                  );
                  // Don't close the modal
                }}
                className="px-5 py-2.5 bg-green-500 text-white rounded-md hover:bg-green-600 flex-1 mr-2"
              >
                Open Agent
              </button>
              <button
                onClick={onClose}
                className="px-5 py-2.5 bg-gray-500 text-white rounded-md hover:bg-gray-600 flex-1"
              >
                Close
              </button>
            </div>

            <h3 className="text-lg font-medium mb-3 text-textStandard">Edit Instructions:</h3>
            <div className="mb-4">
              <div className="border border-borderSubtle rounded-md bg-transparent max-h-[120px] overflow-y-auto">
                <textarea
                  id="instructions"
                  value={instructions}
                  onChange={(e) => setInstructions(e.target.value)}
                  className="w-full p-3 bg-transparent text-textStandard focus:outline-none"
                  placeholder="Instructions for the agent..."
                />
              </div>
            </div>

            {/* Activities Section */}
            <div className="mb-4">
              <label className="block font-medium mb-1 text-textStandard">Activities:</label>
              <div className="border border-borderSubtle rounded-md bg-transparent max-h-[120px] overflow-y-auto mb-2">
                <ul className="divide-y divide-borderSubtle">
                  {activities.map((activity, index) => (
                    <li key={index} className="flex items-center">
                      <span className="flex-1 p-2 text-textStandard">{activity}</span>
                      <button
                        onClick={() => handleRemoveActivity(index)}
                        className="p-1 bg-red-500 text-white rounded-md hover:bg-red-600 m-1"
                      >
                        ✕
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
              <div className="flex">
                <input
                  type="text"
                  value={activityInput}
                  onChange={(e) => setActivityInput(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleAddActivity()}
                  className="flex-1 p-2 border border-borderSubtle rounded-l-md bg-transparent text-textStandard focus:border-borderStandard hover:border-borderStandard"
                  placeholder="Add new activity..."
                />
                <button
                  onClick={handleAddActivity}
                  className="p-2 bg-green-500 text-white rounded-r-md hover:bg-green-600"
                >
                  +
                </button>
              </div>
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}
