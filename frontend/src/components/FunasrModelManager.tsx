import React, { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { toast } from 'sonner';
import { FunasrAPI, FunasrDownloadProgress } from '@/lib/funasr';

interface FunasrModelEntry {
  name: string;
  path: string;
  valid: boolean;
}

interface FunasrModelManagerProps {
  selectedModel?: string;
  onModelSelect?: (modelName: string) => void;
  autoSave?: boolean;
}

export function FunasrModelManager({
  selectedModel,
  onModelSelect,
  autoSave = false
}: FunasrModelManagerProps) {
  const [models, setModels] = useState<FunasrModelEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadStage, setDownloadStage] = useState('');

  // Scan for FunASR models on mount
  useEffect(() => {
    let cancelled = false;

    const scanModels = async () => {
      try {
        setLoading(true);
        setError(null);
        const result = await invoke<FunasrModelEntry[]>('api_scan_funasr_models');
        if (!cancelled) {
          setModels(result);
        }
      } catch (err) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : 'Failed to scan FunASR models';
          console.error('FunASR scan error:', err);
          setError(message);
          toast.error('Failed to scan FunASR models', {
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

  // Listen for download progress events
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      unlisten = await listen<FunasrDownloadProgress>(
        'funasr:download-progress',
        (event) => {
          setDownloadProgress(event.payload.progress);
          setDownloadStage(event.payload.status);
        }
      );
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleDownload = useCallback(async () => {
    setIsDownloading(true);
    setDownloadProgress(0);
    setDownloadStage('');

    try {
      await FunasrAPI.downloadModel();
      // Download complete — rescan models
      const result = await invoke<FunasrModelEntry[]>('api_scan_funasr_models');
      setModels(result);
      toast.success('FunASR model downloaded successfully', {
        description: 'Paraformer model is ready to use',
        duration: 4000
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Download failed';
      toast.error('Failed to download FunASR model', {
        description: message,
        duration: 5000
      });
    } finally {
      setIsDownloading(false);
      setDownloadProgress(0);
      setDownloadStage('');
    }
  }, []);

  const handleCancelDownload = useCallback(async () => {
    try {
      await FunasrAPI.cancelDownload();
      toast.info('Download cancelled', { duration: 3000 });
    } catch (err) {
      console.error('Failed to cancel download:', err);
    } finally {
      setIsDownloading(false);
      setDownloadProgress(0);
      setDownloadStage('');
    }
  }, []);

  const selectModel = useCallback(async (modelName: string) => {
    if (onModelSelect) {
      onModelSelect(modelName);
    }

    if (autoSave) {
      try {
        await invoke('api_save_transcript_config', {
          provider: 'funasr',
          model: modelName,
          apiKey: null as string | null
        });
      } catch (err) {
        console.error('Failed to save FunASR model selection:', err);
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
        <p className="text-sm text-red-800">Failed to scan FunASR models</p>
        <p className="text-xs text-red-600 mt-1">{error}</p>
        <button
          onClick={() => {
            setLoading(true);
            setError(null);
            invoke<FunasrModelEntry[]>('api_scan_funasr_models')
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
              <FunasrModelCard
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
            <span className="text-xl">📁</span>
            <h3 className="font-semibold text-gray-900">No FunASR Models Found</h3>
          </div>
          <p className="text-sm text-gray-600 mb-3">
            Place FunASR model files in your app data directory to use them:
          </p>
          <div className="bg-white rounded border border-gray-200 p-3 font-mono text-xs text-gray-700 space-y-1">
            <p>$APPDATA/Meetily/models/funasr/{'{model_name}'}/</p>
            <p className="text-green-600 ml-4">├── model.onnx</p>
            <p className="text-green-600 ml-4">├── tokens.json</p>
            <p className="text-green-600 ml-4">└── am.mvn</p>
          </div>
          <p className="text-xs text-gray-500 mt-2">
            Supported models: paraformer, paraformer-large, paraformer-8k, etc.
          </p>

          {/* Download section */}
          {!isDownloading ? (
            <button
              onClick={handleDownload}
              disabled={isDownloading}
              className="mt-3 w-full bg-blue-600 text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Download Paraformer Model (~700 MB)
            </button>
          ) : (
            <div className="mt-3 space-y-2">
              <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
                <motion.div
                  className="h-full bg-gradient-to-r from-blue-500 to-blue-600 rounded-full"
                  initial={{ width: 0 }}
                  animate={{ width: `${downloadProgress}%` }}
                  transition={{ duration: 0.3, ease: 'easeOut' }}
                />
              </div>
              <div className="flex items-center justify-between">
                <p className="text-xs text-gray-500">
                  {downloadStage || 'Downloading...'}
                </p>
                <span className="text-xs font-semibold text-blue-600">
                  {Math.round(downloadProgress)}%
                </span>
              </div>
              <button
                onClick={handleCancelDownload}
                className="text-xs text-gray-600 hover:text-red-600 font-medium transition-colors"
              >
                Cancel download
              </button>
            </div>
          )}

          <button
            onClick={() => {
              setLoading(true);
              invoke<FunasrModelEntry[]>('api_scan_funasr_models')
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
        <FunasrModelCard
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
            <FunasrModelCard
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
interface FunasrModelCardProps {
  model: FunasrModelEntry;
  isSelected: boolean;
  onSelect: () => void;
}

function FunasrModelCard({
  model,
  isSelected,
  onSelect
}: FunasrModelCardProps) {
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
              <span className="text-2xl">🎯</span>
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
                Files: model.onnx, token.json, am.mvn
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
              Missing required files: model.onnx, tokens.json, am.mvn
            </p>
          </div>
        )}
      </div>
    </motion.div>
  );
}
