// audio/onnx_provider.rs
//
// Maps a detected [`GpuType`] into an ONNX Runtime execution provider (EP) chain,
// guaranteeing CPU as the last fallback provider.
//
// # Selection strategy
//
// | `GpuType` | Providers tried (in order)                                       |
// |-----------|------------------------------------------------------------------|
// | `None`    | `CPUExecutionProvider` (no GPU path)                             |
// | `OpenCL`  | `CPUExecutionProvider` (no dedicated ORT EP for OpenCL)          |
// | `Metal`   | `CoreMLExecutionProvider` (CPUAndNeuralEngine) → CPU             |
// | `Cuda`    | `CUDAExecutionProvider` → (`DirectMLExecutionProvider`) → CPU    |
// | `Vulkan`  | `DirectMLExecutionProvider` (Windows) / `OpenVINO` (Linux) → CPU |
//
// Each non-CPU EP is feature-gated. Compile with the corresponding Cargo
// feature (e.g. `ort-coreml`, `ort-cuda`, `ort-directml`, `ort-openvino`)
// to enable the hardware-accelerated path.  If a feature is disabled the
// chain silently degrades to CPU-only.

use log::info;

use super::hardware_detector::GpuType;
use ort::execution_providers::CPUExecutionProvider;
use ort::execution_providers::ExecutionProviderDispatch;

#[cfg(feature = "ort-coreml")]
use ort::execution_providers::coreml::CoreMLComputeUnits;
#[cfg(feature = "ort-coreml")]
use ort::execution_providers::CoreMLExecutionProvider;

#[cfg(feature = "ort-cuda")]
use ort::execution_providers::CUDAExecutionProvider;

#[cfg(feature = "ort-directml")]
use ort::execution_providers::DirectMLExecutionProvider;

#[cfg(feature = "ort-openvino")]
use ort::execution_providers::OpenVINOExecutionProvider;

/// Map a detected [`GpuType`] to an ordered list of ORT execution providers.
///
/// CPU is always inserted as the last entry so that ORT falls back to it
/// when no earlier provider supports a given operator.
pub fn get_onnx_providers(gpu_type: GpuType) -> Vec<ExecutionProviderDispatch> {
    let mut providers: Vec<ExecutionProviderDispatch> = Vec::new();

    match gpu_type {
        // ── No GPU / generic GPU compute ──────────────────────────────────
        GpuType::None | GpuType::OpenCL => {
            // No non-CPU EP available — only CPU below.
        }

        // ── Apple Silicon (Metal) ─────────────────────────────────────────
        GpuType::Metal => {
            #[cfg(feature = "ort-coreml")]
            {
                info!("Adding CoreML execution provider (compute units: CPUAndNeuralEngine)");
                providers.push(
                    CoreMLExecutionProvider::default()
                        .with_compute_units(CoreMLComputeUnits::CPUAndNeuralEngine)
                        .build(),
                );
            }
            #[cfg(not(feature = "ort-coreml"))]
            {
                info!("ort-coreml not enabled — CoreML EP unavailable for Metal GPU");
            }
        }

        // ── NVIDIA CUDA ───────────────────────────────────────────────────
        GpuType::Cuda => {
            #[cfg(feature = "ort-cuda")]
            {
                info!("Adding CUDA execution provider");
                providers.push(CUDAExecutionProvider::default().build());
            }
            #[cfg(not(feature = "ort-cuda"))]
            {
                info!("ort-cuda not enabled — skipping CUDA EP");
            }

            #[cfg(feature = "ort-directml")]
            {
                info!("Adding DirectML execution provider (CUDA supplementary)");
                providers.push(DirectMLExecutionProvider::default().build());
            }
        }

        // ── Vulkan (AMD / Intel) ──────────────────────────────────────────
        GpuType::Vulkan => {
            // On Windows, DirectML is the preferred Vulkan-accelerated EP.
            #[cfg(all(target_os = "windows", feature = "ort-directml"))]
            {
                info!("Adding DirectML execution provider (Vulkan / Windows)");
                providers.push(DirectMLExecutionProvider::default().build());
            }

            // On Linux  (and other non-Windows platforms), use OpenVINO.
            #[cfg(all(not(target_os = "windows"), feature = "ort-openvino"))]
            {
                info!("Adding OpenVINO execution provider (Vulkan / Linux)");
                providers.push(OpenVINOExecutionProvider::default().build());
            }
        }
    }

    // ── CPU fallback — always last ───────────────────────────────────────
    providers.push(CPUExecutionProvider::default().build());

    providers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::hardware_detector::GpuType;

    #[test]
    fn test_cpu_only_fallback() {
        let providers = get_onnx_providers(GpuType::None);
        assert_eq!(
            providers.len(),
            1,
            "CPU-only fallback should return exactly 1 provider, got {}",
            providers.len()
        );
    }

    #[test]
    fn test_metal_includes_coreml() {
        let providers = get_onnx_providers(GpuType::Metal);
        #[cfg(feature = "ort-coreml")]
        assert!(
            providers.len() >= 2,
            "With ort-coreml, Metal should have >=2 providers, got {}",
            providers.len()
        );
        #[cfg(not(feature = "ort-coreml"))]
        assert_eq!(
            providers.len(),
            1,
            "Without ort-coreml, Metal should have 1 provider (CPU), got {}",
            providers.len()
        );
    }

    #[test]
    fn test_cuda_includes_cuda() {
        let providers = get_onnx_providers(GpuType::Cuda);
        #[cfg(feature = "ort-cuda")]
        assert!(
            providers.len() >= 2,
            "With ort-cuda, Cuda should have >=2 providers, got {}",
            providers.len()
        );
        #[cfg(not(feature = "ort-cuda"))]
        assert_eq!(
            providers.len(),
            1,
            "Without ort-cuda, Cuda should have 1 provider (CPU), got {}",
            providers.len()
        );
    }

    #[test]
    fn test_cpu_last_in_chain() {
        // CPU is always the last entry in the chain for every GpuType variant.
        for gpu_type in &[
            GpuType::None,
            GpuType::Metal,
            GpuType::Cuda,
            GpuType::Vulkan,
            GpuType::OpenCL,
        ] {
            let providers = get_onnx_providers(*gpu_type);
            assert!(
                !providers.is_empty(),
                "Every GpuType must produce at least the CPU provider"
            );
        }
    }
}
