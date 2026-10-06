#pragma once

#include <cstdint>
#include <memory>

#include "rust/cxx.h"

namespace limo_cad_occt {

struct FfiJob;
struct FfiMesh;
struct FfiDrawingProjection;
struct FfiDrawingOptions;
struct FfiBodyPlacement;
struct FfiInterferenceResult;

class Kernel {
 public:
  Kernel();
  ~Kernel();

  void reset();
  void apply_job(const FfiJob& job);
  rust::Vec<std::uint64_t> body_ids() const;
  FfiMesh mesh(std::uint64_t body_id) const;
  FfiMesh section_mesh(std::uint64_t body_id, std::uint8_t axis,
                       double offset, bool keep_positive, double deflection) const;
  FfiMesh mesh_with_deflection(
      std::uint64_t body_id,
      double linear_deflection,
      double angular_deflection) const;
  rust::Vec<std::uint8_t> export_step(
      const rust::Vec<std::uint64_t>& body_ids,
      rust::Str thread_metadata_hex,
      rust::Str occurrence_placements_hex) const;
  FfiDrawingProjection drawing_projection(
      const rust::Vec<std::uint64_t>& body_ids,
      const rust::Vec<FfiBodyPlacement>& occurrences,
      const FfiDrawingOptions& options) const;
  FfiInterferenceResult exact_interference(
      const FfiBodyPlacement& placement_a,
      const FfiBodyPlacement& placement_b) const;

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

std::unique_ptr<Kernel> new_kernel();

}
