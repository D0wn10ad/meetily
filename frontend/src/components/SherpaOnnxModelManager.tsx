import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { toast } from 'sonner';

interface SherpaOnnxModelEntry {
  name: string;
  path: string;
  valid: boolean;
}

interface SherpaOnnxModelManagerProps {
  selectedModel?: string;
  onModelSelect?: (modelName: string) => void;
  autoSave?: boolean;
}

export function SherpaOnnxModelManager({
  selectedModel,
  onModelSelect,
  autoSave = false
}: SherpaOnnxModelManagerProps) {
  const [models, setModels] = useState<SherpaOnnxModelEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Scan for sherpa-onnx models on mount
  useEffect(() => {
    let cancelled = false;

    const scanModels = async () => {
      try {
        setLoading(true);
        setError(null);
        const result = await invoke<SherpaOnnxModelEntry[]>('sherpa_onnx_scan_models');
        if (!cancelled) {
          setModels(result);
        }
      } catch (err) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : 'Failed to scan sherpa-onnx models';
          console.error('sherpa-onnx scan error:', err);
          setError(message);
          toast.error('Failed to scan sherpa-onnx models', {
            description: message,
            duration: 5000
          });
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };

    scanModels();

    return () => {
      cancelled = true;
    };
  }, []);

  const selectModel = useCallback(async (modelName: string) => {
    if (onModelSelect) {
      onModelSelect(modelName);
    }

    if (autoSave) {
      try {
        await invoke('api_save_transcript_config', {
          provider: 'sherpa-onnx',
          model: modelName,
          apiKey: null as string | null
        });
      } catch (err) {
        console.error('Failed to save sherpa-onnx model selection:', err);
      }
    }

    toast.success(`Switched to ${modelName}`, {
      duration: 3000
    });
  }, [onModelSelect, autoSave]);

  // Loading state
  if (loading) {
    return (
      <div className="space-y-3">
        <div className="animate-pulse space-y-3">
          <div className="h-20 bg-gray-100 rounded-lg"></div>
          <div className="h-20 bg-gray-100 rounded-lg"></div>
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <p className="text-sm text-red-800">Failed to scan sherpa-onnx models</p>
        <p className="text-xs text-red-600 mt-1">{error}</p>
        <button
          onClick={() => {
            setLoading(true);
            setError(null);
            invoke<SherpaOnnxModelEntry[]>('sherpa_onnx_scan_models')
              .then(setModels)
              .catch((err) => setError(err instanceof Error ? err.message : 'Retry failed'))
              .finally(() => setLoading(false));
          }}
          className="mt-3 text-sm text-red-700 underline hover:text-red-800"
        >
          Retry scan
        </button>
      </div>
    );
  }

  // Valid models
  const validModels = models.filter(m => m.valid);
  const invalidModels = models.filter(m => !m.valid);

  // No valid models found — show help card
  if (validModels.length === 0) {
    return (
      <div>
        {/* Show invalid models that exist but are broken */}
        {invalidModels.length > 0 && (
          <div className="space-y-3 mb-4">
            {invalidModels.map(model => (
              <SherpaOnnxModelCard
                key={model.name}
                model={model}
                isSelected={false}
                onSelect={() => {}}
              />
            ))}
          </div>
        )}

        {/* Help card */}
        <motion.div
          initial={{ opacity: 0, y: 5 }}
          animate={{ opacity: 1, y: 0 }}
          className="rounded-lg border-2 border-dashed border-gray-300 bg-gray-50 p-4"
        >
          <div className="flex items-center gap-2 mb-2">
            <span className="text-xl">🎤</span>
            <h3 className="font-semibold text-gray-900">No sherpa-onnx Models Found</h3>
          </div>
          <p className="text-sm text-gray-600 mb-3">
            Place sherpa-onnx model files in your app data directory to use them:
          </p>
          <div className="bg-white rounded border border-gray-200 p-3 font-mono text-xs text-gray-700 space-y-1">
            <p>$APPDATA/Meetily/models/sherpa-onnx/{'{model_name}'}/</p>
            <p className="text-green-600 ml-4">├── model.onnx</p>
            <p className="text-green-600 ml-4">└── tokens.txt</p>
          </div>
          <p className="text-xs text-gray-500 mt-2">
            Supported models: sherpa-onnx-sensevoice-*-encoder-int8, etc.
          </p>
          <button
            onClick={() => {
              setLoading(true);
              invoke<SherpaOnnxModelEntry[]>('sherpa_onnx_scan_models')
                .then(setModels)
                .catch((err) => setError(err instanceof Error ? err.message : 'Scan failed'))
                .finally(() => setLoading(false));
            }}
            className="mt-3 text-sm text-blue-600 hover:text-blue-800 underline"
          >
            Rescan models
          </button>
        </motion.div>
      </div>
    );
  }

  // Models found — render cards
  return (
    <div className="space-y-3">
      {validModels.map(model => (
        <SherpaOnnxModelCard
          key={model.name}
          model={model}
          isSelected={selectedModel === model.name}
          onSelect={() => selectModel(model.name)}
        />
      ))}

      {/* Invalid models shown without select ability */}
      {invalidModels.length > 0 && (
        <div className="space-y-3">
          <div className="border-t border-gray-200 pt-3">
            <p className="text-xs text-gray-500 mb-2">Incomplete models (missing files):</p>
          </div>
          {invalidModels.map(model => (
            <SherpaOnnxModelCard
              key={model.name}
              model={model}
              isSelected={false}
              onSelect={() => {}}
            />
          ))}
        </div>
      )}

      {/* Helper text */}
      {selectedModel && validModels.some(m => m.name === selectedModel) && (
        <motion.div
          initial={{ opacity: 0, y: -5 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-xs text-gray-500 text-center pt-2"
        >
          Using {selectedModel} for transcription
        </motion.div>
      )}
    </div>
  );
}

// Model Card Component
interface SherpaOnnxModelCardProps {
  model: SherpaOnnxModelEntry;
  isSelected: boolean;
  onSelect: () => void;
}

function SherpaOnnxModelCard({
  model,
  isSelected,
  onSelect
}: SherpaOnnxModelCardProps) {
  const [isHovered, setIsHovered] = useState(false);

  const isValid = model.valid;

  return (
    <motion.div
      initial={{ opacity: 0, y: 5 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      className={`
        relative rounded-lg border-2 transition-all
        ${isSelected && isValid
          ? 'border-blue-500 bg-blue-50'
          : isValid
            ? 'border-gray-200 hover:border-gray-300 bg-white cursor-pointer'
            : 'border-gray-200 bg-gray-50'
        }
      `}
      onClick={() => {
        if (isValid) onSelect();
      }}
    >
      <div className="p-4">
        <div className="flex items-start justify-between mb-3">
          <div className="flex-1">
            {/* Model Name */}
            <div className="flex items-center gap-2 mb-1">
              <span className="text-2xl">🧠</span>
              <h3 className="font-semibold text-gray-900">{model.name}</h3>
              {isSelected && isValid && (
                <motion.span
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  className="bg-blue-600 text-white px-2 py-0.5 rounded-full text-xs font-medium flex items-center gap-1"
                >
                  ✓
                </motion.span>
              )}
            </div>

            {/* Model Path */}
            <p className="text-xs text-gray-500 ml-9 font-mono truncate" title={model.path}>
              📁 {model.path}
            </p>

            {/* Required files hint */}
            {isValid && (
              <p className="text-xs text-gray-400 ml-9 mt-1">
                Files: model.onnx, tokens.txt
              </p>
            )}
          </div>

          {/* Status/Action */}
          <div className="ml-4 flex items-center gap-2">
            {isValid ? (
              <>
                <div className="flex items-center gap-1.5 text-green-600">
                  <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                  <span className="text-xs font-medium">Ready</span>
                </div>
                {!isSelected && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onSelect();
                    }}
                    className="bg-blue-600 text-white px-3 py-1.5 rounded-md text-sm font-medium hover:bg-blue-700 transition-colors"
                  >
                    Select
                  </button>
                )}
              </>
            ) : (
              <div className="flex items-center gap-1.5 text-red-600">
                <div className="w-2 h-2 bg-red-500 rounded-full"></div>
                <span className="text-xs font-medium">Missing files</span>
              </div>
            )}
          </div>
        </div>

        {/* Incomplete files hint for invalid models */}
        {!isValid && (
          <div className="mt-2 pt-2 border-t border-gray-200">
            <p className="text-xs text-red-600">
              Missing required files: model.onnx, tokens.txt
            </p>
          </div>
        )}
      </div>
    </motion.div>
  );
}
