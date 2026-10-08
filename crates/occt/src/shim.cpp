#include "limo-cad-occt/src/native.rs.h"

#include <APIHeaderSection_MakeHeader.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepAlgoAPI_Section.hxx>
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepBndLib.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepBuilderAPI_TransitionMode.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepFill.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepMesh_Context.hxx>
#include <BRepMesh_EdgeDiscret.hxx>
#include <BRepMesh_FaceChecker.hxx>
#include <BRepMesh_GeomTool.hxx>
#include <BRepMesh_SphereRangeSplitter.hxx>
#include <BRepMesh_DelabellaMeshAlgoFactory.hxx>
#include <IMeshTools_MeshAlgo.hxx>
#include <IMeshData_Model.hxx>
#include <IMeshData_Face.hxx>
#include <IMeshData_Wire.hxx>
#include <IMeshData_Edge.hxx>
#include <ElCLib.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepLib.hxx>
#include <BRepOffsetAPI_MakePipeShell.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeHalfSpace.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <Bnd_Box.hxx>
#include <GeomAbs_CurveType.hxx>
#include <GeomAbs_Shape.hxx>
#include <GeomAbs_SurfaceType.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <GeomConvert.hxx>
#include <GC_MakeSegment.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <GCPnts_UniformDeflection.hxx>
#include <CPnts_UniformDeflection.hxx>
#include <Precision.hxx>
#include <GC_MakeArcOfCircle.hxx>
#include <GProp_GProps.hxx>
#include <HLRAlgo_Projector.hxx>
#include <HLRBRep_Algo.hxx>
#include <HLRBRep_HLRToShape.hxx>
#include <Message_ProgressRange.hxx>
#include <Message_ProgressIndicator.hxx>
#include <Message_ProgressScope.hxx>
#include <Message.hxx>
#include <Message_Messenger.hxx>
#include <Message_PrinterOStream.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <Interface_Static.hxx>
#include <Interface_HArray1OfHAsciiString.hxx>
#include <Poly_Triangulation.hxx>
#include <OSD_Environment.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Reader.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <STEPControl_Writer.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeFix_Solid.hxx>
#include <StepData_StepModel.hxx>
#include <TCollection_HAsciiString.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_ListIteratorOfListOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Quaternion.hxx>
#include <gp_Sphere.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>

#include <algorithm>
#include <chrono>
#include <array>
#include <cmath>
#include <optional>
#include <cstdint>
#include <cstring>
#include <map>
#include <limits>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace limo_cad_occt {
namespace {

constexpr double kPi = 3.14159265358979323846;
constexpr double kTau = kPi * 2.0;

class SectionProgress final : public Message_ProgressIndicator {
 public:
  explicit SectionProgress(std::uint64_t timeout_ms)
      : deadline_(std::chrono::steady_clock::now() +
                  std::chrono::milliseconds(timeout_ms)) {}
  void check(const char* stage) const {
    if (expired()) {
      throw std::runtime_error(std::string("Section inspection timed out during ") + stage);
    }
  }
 protected:
  Standard_Boolean UserBreak() override { return expired(); }
  void Show(const Message_ProgressScope&, Standard_Boolean) override {}
 private:
  bool expired() const { return std::chrono::steady_clock::now() >= deadline_; }
  const std::chrono::steady_clock::time_point deadline_;
};

struct SectionMeshBudget {
  std::size_t vertices;
  std::size_t edge_points;
  const SectionProgress* progress;
};

double bounded_through_depth(const TopoDS_Shape& shape, double margin) {
  Bnd_Box bounds;
  BRepBndLib::Add(shape, bounds);
  if (bounds.IsVoid()) {
    throw std::runtime_error("could not bound the through-hole target");
  }
  double x_min = 0.0;
  double y_min = 0.0;
  double z_min = 0.0;
  double x_max = 0.0;
  double y_max = 0.0;
  double z_max = 0.0;
  bounds.Get(x_min, y_min, z_min, x_max, y_max, z_max);
  const double diagonal =
      std::hypot(std::hypot(x_max - x_min, y_max - y_min), z_max - z_min);
  if (!std::isfinite(diagonal) || diagonal <= 0.0) {
    throw std::runtime_error("through-hole target bounds are degenerate");
  }
  return diagonal + std::max(margin, 1.0);
}

double bounded_directional_depth(
    const TopoDS_Shape& shape,
    const gp_Pnt& origin,
    const gp_Vec& unit_direction) {
  Bnd_Box bounds;
  BRepBndLib::Add(shape, bounds);
  if (bounds.IsVoid()) {
    throw std::runtime_error("could not bound the threaded-hole target");
  }
  double x_min = 0.0;
  double y_min = 0.0;
  double z_min = 0.0;
  double x_max = 0.0;
  double y_max = 0.0;
  double z_max = 0.0;
  bounds.Get(x_min, y_min, z_min, x_max, y_max, z_max);
  double depth = 0.0;
  for (const double x : {x_min, x_max}) {
    for (const double y : {y_min, y_max}) {
      for (const double z : {z_min, z_max}) {
        depth = std::max(
            depth, gp_Vec(origin, gp_Pnt(x, y, z)).Dot(unit_direction));
      }
    }
  }
  if (!std::isfinite(depth) || depth <= 0.0) {
    throw std::runtime_error("threaded-hole target depth is degenerate");
  }
  return depth;
}

gp_Pnt helical_point(
    const gp_Ax2& axis,
    double radius,
    double center,
    double angle) {
  const gp_Vec radial =
      gp_Vec(axis.XDirection())
          .Multiplied(std::cos(angle))
          .Added(gp_Vec(axis.YDirection()).Multiplied(std::sin(angle)));
  return axis.Location().Translated(
      radial.Multiplied(radius).Added(
          gp_Vec(axis.Direction()).Multiplied(center)));
}

TopoDS_Edge make_helical_edge(
    double center_start,
    double center_end,
    double pitch,
    bool left_hand,
    const Handle(Geom_CylindricalSurface)& surface,
    double curve_tolerance = 1e-5) {
  const double angle_span = kTau * (center_end - center_start) / pitch;
  const double axial_per_radian = pitch / kTau;
  const double handedness = left_hand ? -1.0 : 1.0;
  const double parameter_span =
      angle_span * std::hypot(1.0, axial_per_radian);
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(
      gp_Pnt2d(0.0, center_start),
      gp_Dir2d(handedness, axial_per_radian));
  BRepBuilderAPI_MakeEdge builder(pcurve, surface, 0.0, parameter_span);
  if (!builder.IsDone()) {
    throw std::runtime_error("OCCT could not build the exact helical edge");
  }
  TopoDS_Edge edge = builder.Edge();
  BRepLib::BuildCurve3d(edge, curve_tolerance);
  BRepLib::SameParameter(edge, 1e-7);
  return edge;
}




TopoDS_Face make_curved_helical_face(
    const gp_Ax2& axis,
    const Handle(Geom_BSplineCurve)& helix,
    const Handle(Geom_Curve)& section) {
  const Handle(Geom_BSplineCurve) profile =
      GeomConvert::CurveToBSplineCurve(section);
  TColgp_Array2OfPnt poles(1, helix->NbPoles(), 1, profile->NbPoles());
  TColStd_Array2OfReal weights(1, helix->NbPoles(), 1, profile->NbPoles());
  for (int u = 1; u <= helix->NbPoles(); ++u) {
    const gp_Pnt h = helix->Pole(u);
    for (int v = 1; v <= profile->NbPoles(); ++v) {
      const gp_Pnt p = profile->Pole(v);
      poles.SetValue(u, v, axis.Location().Translated(
          gp_Vec(axis.XDirection()).Multiplied(h.X() * p.X())
              .Added(gp_Vec(axis.YDirection()).Multiplied(h.Y() * p.X()))
              .Added(gp_Vec(axis.Direction()).Multiplied(h.Z() + p.Z()))));
      weights.SetValue(u, v, helix->Weight(u) * profile->Weight(v));
    }
  }
  TColStd_Array1OfReal u_knots(1, helix->NbKnots());
  TColStd_Array1OfReal v_knots(1, profile->NbKnots());
  TColStd_Array1OfInteger u_mults(1, helix->NbKnots());
  TColStd_Array1OfInteger v_mults(1, profile->NbKnots());
  helix->Knots(u_knots); helix->Multiplicities(u_mults);
  profile->Knots(v_knots); profile->Multiplicities(v_mults);
  Handle(Geom_BSplineSurface) surface = new Geom_BSplineSurface(
      poles, weights, u_knots, v_knots, u_mults, v_mults,
      helix->Degree(), profile->Degree());
  BRepBuilderAPI_MakeFace face(surface, 1e-7);
  if (!face.IsDone()) throw std::runtime_error("could not build rounded helical face");
  return face.Face();
}

TopoDS_Shape make_continuous_thread_cutter(
    const gp_Ax2& axis,
    double spine_radius,
    const std::vector<std::pair<double, double>>& radius_half_widths,
    double pitch,
    double thread_depth,
    bool left_hand,
    const char* label,
    const std::vector<Handle(Geom_Curve)>& curved_profile = {}) {



  const double center_start = -pitch;
  const double center_end = thread_depth + pitch;
  const double turns = (center_end - center_start) / pitch;
  if (!std::isfinite(turns) || turns <= 0.0 || turns > 256.0) {
    throw std::runtime_error(
        std::string(label) +
        " thread interval is too short or exceeds 256 turns; use simplified representation");
  }

  const double inner_radius = radius_half_widths.front().first;
  const double outer_radius = radius_half_widths.back().first;
  if (!std::isfinite(spine_radius) || spine_radius <= inner_radius ||
      spine_radius >= outer_radius) {
    throw std::runtime_error("thread spine must lie inside its radial profile");
  }
  double previous_radius = -1.0;
  for (const auto& station : radius_half_widths) {
    if (!std::isfinite(station.first) || !std::isfinite(station.second) ||
        station.first <= previous_radius || station.second <= 0.0 ||
        station.second >= pitch * 0.5) {
      throw std::runtime_error("thread profile radial stations are invalid");
    }
    previous_radius = station.first;
  }






  BRepBuilderAPI_Sewing sewing(1e-7, true, true, true, false);
  const int segment_count = static_cast<int>(std::ceil(turns - 1e-10));
  for (int segment_index = 0; segment_index < segment_count;
       ++segment_index) {
    const double segment_start = center_start + segment_index * pitch;
    const double segment_end = std::min(segment_start + pitch, center_end);
    if (!curved_profile.empty()) {
      Handle(Geom_CylindricalSurface) unit_surface = new Geom_CylindricalSurface(
          gp_Ax3(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.0);
      const TopoDS_Edge spine = make_helical_edge(
          segment_start, segment_end, pitch, left_hand, unit_surface, 1e-10);
      double first, last;
      Handle(Geom_Curve) curve = BRep_Tool::Curve(spine, first, last);
      Handle(Geom_BSplineCurve) helix = GeomConvert::CurveToBSplineCurve(
          new Geom_TrimmedCurve(curve, first, last));
      for (const auto& section : curved_profile) {
        sewing.Add(make_curved_helical_face(axis, helix, section));
      }
      continue;
    }
    std::vector<TopoDS_Edge> lower_rails;
    std::vector<TopoDS_Edge> upper_rails;
    lower_rails.reserve(radius_half_widths.size());
    upper_rails.reserve(radius_half_widths.size());
    for (const auto& station : radius_half_widths) {
      Handle(Geom_CylindricalSurface) rail_surface =
          new Geom_CylindricalSurface(gp_Ax3(axis), station.first);
      lower_rails.push_back(make_helical_edge(
          segment_start - station.second, segment_end - station.second,
          pitch, left_hand, rail_surface));
      upper_rails.push_back(make_helical_edge(
          segment_start + station.second, segment_end + station.second,
          pitch, left_hand, rail_surface));
    }
    sewing.Add(BRepFill::Face(lower_rails.front(), upper_rails.front()));
    sewing.Add(BRepFill::Face(lower_rails.back(), upper_rails.back()));
    for (std::size_t index = 0; index + 1 < lower_rails.size(); ++index) {
      sewing.Add(BRepFill::Face(lower_rails[index], lower_rails[index + 1]));
      sewing.Add(BRepFill::Face(upper_rails[index], upper_rails[index + 1]));
    }
  }

  const double handedness = left_hand ? -1.0 : 1.0;
  const double end_angle = handedness * kTau * turns;
  const auto make_cap = [&](double center, double angle) {
    if (!curved_profile.empty()) {
      const gp_Vec x = gp_Vec(axis.XDirection()).Multiplied(std::cos(angle))
          .Added(gp_Vec(axis.YDirection()).Multiplied(std::sin(angle)));
      const gp_Vec y = gp_Vec(axis.XDirection()).Multiplied(-std::sin(angle))
          .Added(gp_Vec(axis.YDirection()).Multiplied(std::cos(angle)));
      const gp_Vec z(axis.Direction());
      const gp_Pnt origin = axis.Location().Translated(z.Multiplied(center));
      gp_Trsf placement;
      placement.SetValues(x.X(), y.X(), z.X(), origin.X(),
                          x.Y(), y.Y(), z.Y(), origin.Y(),
                          x.Z(), y.Z(), z.Z(), origin.Z());
      BRepBuilderAPI_MakeWire wire;
      for (const auto& section : curved_profile) {
        Handle(Geom_Curve) placed = Handle(Geom_Curve)::DownCast(section->Transformed(placement));
        wire.Add(BRepBuilderAPI_MakeEdge(placed).Edge());
      }
      BRepBuilderAPI_MakeFace face(wire.Wire(), true);
      if (!face.IsDone()) throw std::runtime_error("could not cap rounded thread");
      return face.Face();
    }
    BRepBuilderAPI_MakePolygon polygon;
    for (const auto& station : radius_half_widths) {
      polygon.Add(helical_point(
          axis, station.first, center - station.second, angle));
    }
    for (auto station = radius_half_widths.rbegin();
         station != radius_half_widths.rend(); ++station) {
      polygon.Add(helical_point(
          axis, station->first, center + station->second, angle));
    }
    polygon.Close();
    if (!polygon.IsDone()) {
      throw std::runtime_error(
          std::string("OCCT could not close the continuous ") + label +
          " thread cutter end");
    }
    BRepBuilderAPI_MakeFace face(polygon.Wire(), true);
    if (!face.IsDone()) {
      throw std::runtime_error(
          std::string("OCCT could not cap the continuous ") + label +
          " thread cutter");
    }
    return face.Face();
  };
  sewing.Add(make_cap(center_start, 0.0));
  sewing.Add(make_cap(center_end, end_angle));
  sewing.Perform(Message_ProgressRange());
  if (sewing.SewedShape().IsNull() || sewing.NbFreeEdges() != 0 ||
      sewing.NbMultipleEdges() != 0) {
    throw std::runtime_error(
        std::string("OCCT could not sew the continuous ") + label +
        " thread cutter into a closed shell (free edges=" +
        std::to_string(sewing.NbFreeEdges()) + ", multiple edges=" +
        std::to_string(sewing.NbMultipleEdges()) + ")");
  }
  TopExp_Explorer shells(sewing.SewedShape(), TopAbs_SHELL);
  if (!shells.More()) {
    throw std::runtime_error(
        std::string("OCCT continuous ") + label +
        " thread boundary did not produce a shell");
  }
  const TopoDS_Shell shell = TopoDS::Shell(shells.Current());
  shells.Next();
  if (shells.More()) {
    throw std::runtime_error(
        std::string("OCCT continuous ") + label +
        " thread boundary produced multiple shells");
  }
  ShapeFix_Solid solid_fixer;
  solid_fixer.SetPrecision(1e-7);
  TopoDS_Solid cutter = solid_fixer.SolidFromShell(shell);
  if (cutter.IsNull()) {
    throw std::runtime_error(
        std::string("OCCT could not solidify the continuous ") + label +
        " thread boundary");
  }
  if (!BRepLib::OrientClosedSolid(cutter)) {
    throw std::runtime_error(
        std::string("OCCT could not orient the continuous ") + label +
        " thread cutter solid");
  }
  BRepLib::SameParameter(cutter, 1e-6, true);

  const double sample_center = (center_start + center_end) * 0.5;
  const double sample_angle =
      (left_hand ? -1.0 : 1.0) * kTau *
      (sample_center - center_start) / pitch;
  const gp_Pnt sample_point = helical_point(
      axis, (inner_radius + outer_radius) * 0.5, sample_center,
      sample_angle);
  const gp_Pnt inner_probe = helical_point(
      axis, inner_radius + (outer_radius - inner_radius) * 0.1,
      sample_center, sample_angle);
  const gp_Pnt outer_probe = helical_point(
      axis, outer_radius - (outer_radius - inner_radius) * 0.1,
      sample_center, sample_angle);
  const double boundary_probe =
      std::max(1e-5, (outer_radius - inner_radius) * 1e-3);
  const gp_Pnt inside_inner_boundary = helical_point(
      axis, inner_radius + boundary_probe, sample_center, sample_angle);
  const gp_Pnt outside_inner_boundary = helical_point(
      axis, inner_radius - boundary_probe, sample_center, sample_angle);
  const gp_Pnt inside_outer_boundary = helical_point(
      axis, outer_radius - boundary_probe, sample_center, sample_angle);
  const gp_Pnt outside_outer_boundary = helical_point(
      axis, outer_radius + boundary_probe, sample_center, sample_angle);
  const gp_Pnt axis_probe = axis.Location().Translated(
      gp_Vec(axis.Direction()).Multiplied(sample_center));
  const auto is_inside = [&](const gp_Pnt& point) {
    BRepClass3d_SolidClassifier classifier(cutter, point, 1e-7);
    return classifier.State() == TopAbs_IN ||
           classifier.State() == TopAbs_ON;
  };



  if (!is_inside(sample_point) || !is_inside(inner_probe) ||
      !is_inside(outer_probe) || !is_inside(inside_inner_boundary) ||
      is_inside(outside_inner_boundary) ||
      !is_inside(inside_outer_boundary) ||
      is_inside(outside_outer_boundary) || is_inside(axis_probe)) {
    Bnd_Box bounds;
    BRepBndLib::Add(cutter, bounds);
    std::ostringstream details;
    if (!bounds.IsVoid()) {
      double x_min = 0.0;
      double y_min = 0.0;
      double z_min = 0.0;
      double x_max = 0.0;
      double y_max = 0.0;
      double z_max = 0.0;
      bounds.Get(x_min, y_min, z_min, x_max, y_max, z_max);
      details << " bounds=[" << x_min << "," << y_min << "," << z_min
              << "]-[" << x_max << "," << y_max << "," << z_max << "]";
    }
    details << " probes=" << is_inside(sample_point) << ","
            << is_inside(inner_probe) << "," << is_inside(outer_probe)
            << ",inner=" << is_inside(outside_inner_boundary) << "/"
            << is_inside(inside_inner_boundary) << ",outer="
            << is_inside(inside_outer_boundary) << "/"
            << is_inside(outside_outer_boundary)
            << ",axis=" << is_inside(axis_probe);
    throw std::runtime_error(
        std::string("OCCT continuous ") + label +
        " thread cutter is inside-out" + details.str());
  }
  BRepCheck_Analyzer analyzer(cutter, true, false);
  if (!analyzer.IsValid()) {
    throw std::runtime_error(
        std::string("OCCT continuous ") + label +
        " thread cutter is invalid");
  }
  GProp_GProps properties;
  BRepGProp::VolumeProperties(cutter, properties);
  if (!std::isfinite(properties.Mass()) ||
      std::abs(properties.Mass()) <= 1e-9) {
    throw std::runtime_error(
        std::string("OCCT continuous ") + label +
        " thread cutter has no volume");
  }
  return cutter;
}

std::vector<TopoDS_Shape> make_rounded_thread_cutters(
    const gp_Ax2& axis, double major, double minor, double pitch,
    double radius, double axial_clearance, double depth, bool left_hand,
    bool internal) {
  const double lo = minor * 0.5;
  const double hi = major * 0.5;
  const double beta = kPi / 12.0;
  const double h0 = pitch * 0.25 - (hi - lo) * 0.5 * std::tan(beta);
  const double h1 = pitch * 0.25 + (hi - lo) * 0.5 * std::tan(beta);
  const double corner = radius * (1.0 / std::cos(beta) - std::tan(beta));
  const double z0 = h0 - corner;
  const double z1 = h1 + corner;
  const double overlap = std::max(0.005, pitch * 0.02);
  if (lo <= overlap || radius <= 0 ||
      2 * radius * (1 - std::sin(beta)) >= hi - lo ||
      z0 <= axial_clearance * 0.5 || z1 + axial_clearance * 0.5 >= pitch * 0.5) {
    throw std::runtime_error("rounded trapezoidal thread profile is invalid");
  }


  const auto point = [&](double r, double z) {
    return gp_Pnt(r, 0, internal ? pitch * 0.5 - z + axial_clearance * 0.5 : z);
  };
  const auto arc_point = [&](double cr, double cz, double angle) {
    return point(cr + radius * std::cos(angle), cz + radius * std::sin(angle));
  };
  const gp_Pnt root = point(lo, z0);
  const gp_Pnt root_tangent = arc_point(lo + radius, z0, kPi * 0.5 + beta);
  const gp_Pnt crest_tangent = arc_point(hi - radius, z1, beta - kPi * 0.5);
  const gp_Pnt crest = point(hi, z1);
  std::vector<Handle(Geom_Curve)> upper;
  if (internal) upper.push_back(GC_MakeSegment(point(lo - overlap, z0), root).Value());
  upper.push_back(GC_MakeArcOfCircle(root,
      arc_point(lo + radius, z0, (kPi + kPi * 0.5 + beta) * 0.5), root_tangent).Value());
  upper.push_back(GC_MakeSegment(root_tangent, crest_tangent).Value());
  upper.push_back(GC_MakeArcOfCircle(crest_tangent,
      arc_point(hi - radius, z1, (beta - kPi * 0.5) * 0.5), crest).Value());
  if (!internal) upper.push_back(GC_MakeSegment(crest, point(hi + overlap, z1)).Value());
  std::vector<Handle(Geom_Curve)> curves = upper;
  const auto mirror = [](const gp_Pnt& p) { return gp_Pnt(p.X(), 0, -p.Z()); };
  const gp_Pnt start = upper.front()->Value(upper.front()->FirstParameter());
  const gp_Pnt end = upper.back()->Value(upper.back()->LastParameter());
  curves.push_back(GC_MakeSegment(end, mirror(end)).Value());
  gp_Trsf reflection;
  reflection.SetMirror(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
  for (auto it = upper.rbegin(); it != upper.rend(); ++it) {
    Handle(Geom_Curve) lower = Handle(Geom_Curve)::DownCast((*it)->Transformed(reflection));
    lower->Reverse();
    curves.push_back(lower);
  }
  curves.push_back(GC_MakeSegment(mirror(start), start).Value());
  const std::vector<std::pair<double, double>> stations = {
      {start.X(), start.Z()}, {end.X(), end.Z()}};
  TopoDS_Shape cutter = make_continuous_thread_cutter(
      axis, (hi + lo) * 0.5, stations, pitch, depth, left_hand,
      "custom rounded trapezoidal", curves);
  return {cutter};
}

void trim_thread_tools_at_depth(
    std::vector<TopoDS_Shape>& cutters, const gp_Ax2& axis,
    double major_radius, double pitch, double depth, bool bound_start = false) {



  const double start_offset = bound_start ? 0.0 : -pitch;
  const gp_Ax2 clip_axis(
      axis.Location().Translated(gp_Vec(axis.Direction()).Multiplied(start_offset)),
      axis.Direction(), axis.XDirection());
  BRepPrimAPI_MakeCylinder clip(clip_axis, major_radius + pitch, depth - start_offset);
  for (TopoDS_Shape& cutter : cutters) {
    BRepAlgoAPI_Common trimmed(cutter, clip.Shape(), Message_ProgressRange());
    if (!trimmed.IsDone() || trimmed.HasErrors() || trimmed.Shape().IsNull()) {
      throw std::runtime_error("could not trim thread to its requested depth");
    }
    cutter = trimmed.Shape();
  }
}

std::vector<TopoDS_Shape> make_internal_thread_cutters(
    const gp_Ax2& axis,
    double major_diameter,
    double pitch_diameter,
    double minor_diameter,
    double pitch,
    double thread_depth,
    bool left_hand) {
  const double overlap = std::max(
      2e-3, std::min({minor_diameter * 5e-3, pitch * 2e-2,
                      (pitch_diameter - minor_diameter) * 2.5e-2}));
  const double minor_radius = minor_diameter * 0.5;
  const double inner_radius = minor_radius - overlap;
  const double pitch_radius = pitch_diameter * 0.5;
  const double outer_radius = major_diameter * 0.5;
  const double pitch_half_width = pitch * 0.25;
  const double outer_half_width =
      pitch_half_width -
      (outer_radius - pitch_radius) * std::tan(kPi / 6.0);
  const double inner_half_width =
      pitch_half_width +
      (pitch_radius - minor_radius) * std::tan(kPi / 6.0);
  if (inner_radius <= 0.0 || pitch_radius <= inner_radius ||
      outer_radius <= pitch_radius || outer_half_width <= 0.0 ||
      inner_half_width >= pitch * 0.499) {
    throw std::runtime_error(
        "ISO internal thread limits do not form a valid 60-degree profile");
  }
  const std::vector<std::pair<double, double>> profile = {
      {inner_radius, inner_half_width},
      {minor_radius, inner_half_width},
      {outer_radius, outer_half_width},
  };
  return {make_continuous_thread_cutter(
      axis, pitch_radius, profile, pitch, thread_depth, left_hand,
      "internal")};
}

std::vector<TopoDS_Shape> make_external_thread_cutters(
    const gp_Ax2& axis,
    double major_diameter,
    double pitch_diameter,
    double minor_diameter,
    double pitch,
    double thread_depth,
    bool left_hand) {
  const double overlap = std::max(
      5e-3, std::min({major_diameter * 3e-2, pitch * 1.5e-1,
                      (major_diameter - pitch_diameter) * 5e-1}));
  const double inner_radius = minor_diameter * 0.5;
  const double pitch_radius = pitch_diameter * 0.5;
  const double major_radius = major_diameter * 0.5;
  const double outer_radius = major_radius + overlap;
  const double pitch_half_width = pitch * 0.25;
  const double inner_half_width =
      pitch_half_width -
      (pitch_radius - inner_radius) * std::tan(kPi / 6.0);
  const double outer_half_width =
      pitch_half_width +
      (major_radius - pitch_radius) * std::tan(kPi / 6.0);
  if (inner_radius <= 0.0 || pitch_radius <= inner_radius ||
      outer_radius <= pitch_radius || inner_half_width <= 0.0 ||
      outer_half_width >= pitch * 0.499) {
    throw std::runtime_error(
        "ISO external thread limits do not form a valid 60-degree profile");
  }
  const std::vector<std::pair<double, double>> profile = {
      {inner_radius, inner_half_width},
      {major_radius, outer_half_width},
      {outer_radius, outer_half_width},
  };
  return {make_continuous_thread_cutter(
      axis, pitch_radius, profile, pitch, thread_depth, left_hand,
      "external")};
}

TopoDS_Shape cut_thread_tools(
    const TopoDS_Shape& target,
    const std::vector<TopoDS_Shape>& cutters) {
  if (cutters.empty()) {
    return target;
  }
  TopTools_ListOfShape arguments;
  arguments.Append(target);
  TopTools_ListOfShape tools;
  for (const TopoDS_Shape& cutter : cutters) {
    tools.Append(cutter);
  }
  BRepAlgoAPI_Cut cut;
  cut.SetArguments(arguments);
  cut.SetTools(tools);
  cut.SetNonDestructive(true);
  cut.SetRunParallel(true);
  cut.Build(Message_ProgressRange());
  if (!cut.IsDone() || cut.HasErrors() || cut.Shape().IsNull()) {
    throw std::runtime_error("OCCT modeled thread cut failed");
  }
  return cut.Shape();
}

gp_Pnt point_at(const FfiJob& job, std::size_t point_index) {
  const std::size_t offset = point_index * 3;
  if (offset + 2 >= job.points.size()) {
    throw std::runtime_error("profile point buffer is malformed");
  }
  return gp_Pnt(job.points[offset], job.points[offset + 1], job.points[offset + 2]);
}

TopoDS_Wire make_wire(const std::vector<gp_Pnt>& points) {
  if (points.size() < 3) {
    throw std::runtime_error("profile must contain at least three points");
  }
  BRepBuilderAPI_MakePolygon polygon;
  for (const gp_Pnt& point : points) {
    polygon.Add(point);
  }
  polygon.Close();
  if (!polygon.IsDone()) {
    throw std::runtime_error("OCCT could not build the profile wire");
  }
  return polygon.Wire();
}

TopoDS_Wire make_open_wire(const std::vector<gp_Pnt>& points) {
  if (points.size() < 2) {
    throw std::runtime_error("path must contain at least two points");
  }
  BRepBuilderAPI_MakePolygon polygon;
  for (const gp_Pnt& point : points) {
    polygon.Add(point);
  }
  if (!polygon.IsDone()) {
    throw std::runtime_error("OCCT could not build the path wire");
  }
  return polygon.Wire();
}

gp_Pnt buffered_curve_point(const rust::Vec<double>& points,
                            std::size_t point_index,
                            const char* label) {
  const std::size_t offset = point_index * 3;
  if (offset + 2 >= points.size()) {
    throw std::runtime_error(std::string(label) + " curve point buffer is malformed");
  }
  return gp_Pnt(points[offset], points[offset + 1], points[offset + 2]);
}

TopoDS_Wire make_curve_wire(const rust::Vec<std::uint8_t>& kinds,
                            const rust::Vec<std::uint32_t>& offsets,
                            const rust::Vec<double>& points,
                            const char* label) {
  if (kinds.empty() || offsets.size() != kinds.size() + 1 ||
      offsets.front() != 0 || offsets.back() * 3 != points.size()) {
    throw std::runtime_error(std::string(label) + " curve buffers are malformed");
  }
  BRepBuilderAPI_MakeWire wire;
  for (std::size_t curve_index = 0; curve_index < kinds.size(); ++curve_index) {
    const std::size_t begin = offsets[curve_index];
    const std::size_t end = offsets[curve_index + 1];
    const std::size_t count = end - begin;
    auto point = [&](std::size_t index) {
      return buffered_curve_point(points, index, label);
    };
    if (kinds[curve_index] == 0) {
      if (count != 2) {
        throw std::runtime_error(std::string(label) + " line needs two points");
      }
      BRepBuilderAPI_MakeEdge edge(point(begin), point(begin + 1));
      if (!edge.IsDone()) {
        throw std::runtime_error(std::string("OCCT could not build the ") + label +
                                 " line");
      }
      wire.Add(edge.Edge());
    } else if (kinds[curve_index] == 1) {
      if (count != 3) {
        throw std::runtime_error(std::string(label) +
                                 " arc needs start/mid/end points");
      }
      GC_MakeArcOfCircle arc(point(begin), point(begin + 1), point(begin + 2));
      if (!arc.IsDone()) {
        throw std::runtime_error(std::string("OCCT could not build the ") + label +
                                 " arc");
      }
      BRepBuilderAPI_MakeEdge edge(arc.Value());
      if (!edge.IsDone()) {
        throw std::runtime_error(std::string("OCCT could not build the ") + label +
                                 " arc edge");
      }
      wire.Add(edge.Edge());
    } else if (kinds[curve_index] == 2) {
      if (count != 3) {
        throw std::runtime_error(std::string(label) +
                                 " circle needs center/axis/normal data");
      }
      const gp_Pnt center = point(begin);
      const gp_Pnt axis_point = point(begin + 1);
      const gp_Pnt normal_data = point(begin + 2);
      const gp_Vec axis(center, axis_point);
      const gp_Vec normal(normal_data.X(), normal_data.Y(), normal_data.Z());
      if (axis.SquareMagnitude() < 1e-18 || normal.SquareMagnitude() < 1e-18) {
        throw std::runtime_error(std::string(label) + " circle axes are degenerate");
      }
      BRepBuilderAPI_MakeEdge edge(
          gp_Circ(gp_Ax2(center, gp_Dir(normal), gp_Dir(axis)), axis.Magnitude()));
      if (!edge.IsDone()) {
        throw std::runtime_error(std::string("OCCT could not build the ") + label +
                                 " circle");
      }
      wire.Add(edge.Edge());
    } else if (kinds[curve_index] == 3) {
      if (count < 2) {
        throw std::runtime_error(std::string(label) +
                                 " polyline needs at least two points");
      }
      for (std::size_t index = begin; index + 1 < end; ++index) {
        BRepBuilderAPI_MakeEdge edge(point(index), point(index + 1));
        if (!edge.IsDone()) {
          throw std::runtime_error(std::string("OCCT could not build the ") +
                                   label + " polyline");
        }
        wire.Add(edge.Edge());
      }
    } else {
      throw std::runtime_error(std::string("unknown ") + label + " curve kind");
    }
  }
  if (!wire.IsDone()) {
    throw std::runtime_error(std::string("OCCT could not build the ") + label +
                             " wire");
  }
  return wire.Wire();
}

struct SectionTransform {
  gp_Pnt centroid;
  gp_Vec translation;
  double scale;

  gp_Pnt Apply(const gp_Pnt& point) const {
    gp_Vec radial(centroid, point);
    radial.Multiply(scale);
    gp_Pnt transformed = centroid.Translated(radial);
    transformed.Translate(translation);
    return transformed;
  }
};

SectionTransform section_transform(const FfiJob& job, std::size_t begin,
                                   std::size_t end, double offset,
                                   double reference_radius) {
  const gp_Vec normal(job.normal_x, job.normal_y, job.normal_z);
  if (normal.SquareMagnitude() < 1e-18) {
    throw std::runtime_error("extrude normal is degenerate");
  }
  gp_Vec unit = normal.Normalized();
  gp_Pnt centroid(0.0, 0.0, 0.0);
  for (std::size_t index = begin; index < end; ++index) {
    const gp_Pnt point = point_at(job, index);
    centroid.SetX(centroid.X() + point.X());
    centroid.SetY(centroid.Y() + point.Y());
    centroid.SetZ(centroid.Z() + point.Z());
  }
  const double count = static_cast<double>(end - begin);
  centroid.SetX(centroid.X() / count);
  centroid.SetY(centroid.Y() / count);
  centroid.SetZ(centroid.Z() / count);

  const double angle = job.taper_angle_deg * kPi / 180.0;
  const double scale = 1.0 + std::tan(angle) * offset / reference_radius;
  if (!std::isfinite(scale) || scale <= 1e-6) {
    throw std::runtime_error("taper collapses or inverts the profile");
  }
  return SectionTransform{centroid, unit.Multiplied(offset), scale};
}

gp_Pnt curve_point_at(const FfiJob& job, std::size_t point_index) {
  const std::size_t offset = point_index * 3;
  if (offset + 2 >= job.curve_points.size()) {
    throw std::runtime_error("profile curve point buffer is malformed");
  }
  return gp_Pnt(job.curve_points[offset], job.curve_points[offset + 1],
                job.curve_points[offset + 2]);
}

TopoDS_Wire make_profile_wire(const FfiJob& job, std::size_t profile_index,
                              const SectionTransform* transform = nullptr) {
  if (profile_index + 1 >= job.profile_offsets.size()) {
    throw std::runtime_error("profile offset buffer is malformed");
  }
  const std::size_t point_begin = job.profile_offsets[profile_index];
  const std::size_t point_end = job.profile_offsets[profile_index + 1];


  if (job.curve_kinds.empty() || job.curve_profile_offsets.empty()) {
    std::vector<gp_Pnt> points;
    points.reserve(point_end - point_begin);
    for (std::size_t index = point_begin; index < point_end; ++index) {
      const gp_Pnt value = point_at(job, index);
      points.push_back(transform == nullptr ? value : transform->Apply(value));
    }
    return make_wire(points);
  }
  if (job.curve_profile_offsets.size() != job.profile_offsets.size() ||
      job.curve_point_offsets.size() != job.curve_kinds.size() + 1 ||
      job.curve_point_offsets.back() * 3 != job.curve_points.size()) {
    throw std::runtime_error("profile curve buffers are malformed");
  }

  const std::size_t curve_begin = job.curve_profile_offsets[profile_index];
  const std::size_t curve_end = job.curve_profile_offsets[profile_index + 1];
  if (curve_end <= curve_begin || curve_end > job.curve_kinds.size()) {
    throw std::runtime_error("profile contains no boundary curves");
  }

  auto transformed = [&](std::size_t point_index) {
    const gp_Pnt value = curve_point_at(job, point_index);
    return transform == nullptr ? value : transform->Apply(value);
  };
  BRepBuilderAPI_MakeWire wire;
  for (std::size_t curve_index = curve_begin; curve_index < curve_end;
       ++curve_index) {
    const std::size_t begin = job.curve_point_offsets[curve_index];
    const std::size_t end = job.curve_point_offsets[curve_index + 1];
    const std::size_t count = end - begin;
    switch (job.curve_kinds[curve_index]) {
      case 0: {
        if (count != 2) {
          throw std::runtime_error("line curve requires two points");
        }
        BRepBuilderAPI_MakeEdge edge(transformed(begin),
                                     transformed(begin + 1));
        if (!edge.IsDone()) {
          throw std::runtime_error("OCCT could not build a line profile edge");
        }
        wire.Add(edge.Edge());
        break;
      }
      case 1: {
        if (count != 3) {
          throw std::runtime_error("arc curve requires start/mid/end points");
        }
        const gp_Pnt start = transformed(begin);
        const gp_Pnt mid = transformed(begin + 1);
        const gp_Pnt finish = transformed(begin + 2);
        GC_MakeArcOfCircle arc(start, mid, finish);
        if (!arc.IsDone()) {
          throw std::runtime_error("OCCT could not build an analytic arc");
        }
        BRepBuilderAPI_MakeEdge edge(arc.Value());
        if (!edge.IsDone()) {
          throw std::runtime_error("OCCT could not build an arc profile edge");
        }
        wire.Add(edge.Edge());
        break;
      }
      case 2: {
        if (count != 3) {
          throw std::runtime_error(
              "circle curve requires center/axis/normal data");
        }
        const gp_Pnt center = transformed(begin);
        const gp_Pnt axis_point = transformed(begin + 1);
        const gp_Pnt normal_data = curve_point_at(job, begin + 2);
        const gp_Vec axis(center, axis_point);
        const gp_Vec normal(normal_data.X(), normal_data.Y(), normal_data.Z());
        if (axis.SquareMagnitude() < 1e-18 ||
            normal.SquareMagnitude() < 1e-18) {
          throw std::runtime_error("circle curve has degenerate axes");
        }
        const gp_Circ circle(gp_Ax2(center, gp_Dir(normal), gp_Dir(axis)),
                             axis.Magnitude());
        BRepBuilderAPI_MakeEdge edge(circle);
        if (!edge.IsDone()) {
          throw std::runtime_error("OCCT could not build a circle profile edge");
        }
        wire.Add(edge.Edge());
        break;
      }
      case 3: {
        if (count < 2) {
          throw std::runtime_error("polyline curve needs at least two points");
        }
        for (std::size_t index = begin; index + 1 < end; ++index) {
          BRepBuilderAPI_MakeEdge edge(transformed(index),
                                       transformed(index + 1));
          if (!edge.IsDone()) {
            throw std::runtime_error(
                "OCCT could not build a polyline profile edge");
          }
          wire.Add(edge.Edge());
        }
        break;
      }
      default:
        throw std::runtime_error("unknown profile curve kind");
    }
  }
  if (!wire.IsDone()) {
    throw std::runtime_error("OCCT could not build the analytic profile wire");
  }
  return wire.Wire();
}

std::pair<std::size_t, std::size_t> region_range(const FfiJob& job,
                                                  std::size_t region_index) {
  if (region_index + 1 >= job.region_offsets.size()) {
    throw std::runtime_error("profile region buffer is malformed");
  }
  const std::size_t begin = job.region_offsets[region_index];
  const std::size_t end = job.region_offsets[region_index + 1];
  if (end <= begin || end >= job.profile_offsets.size()) {
    throw std::runtime_error("profile region is empty or out of range");
  }
  return {begin, end};
}

TopoDS_Face make_profile_face(const FfiJob& job, std::size_t profile_index,
                              const SectionTransform* transform = nullptr) {
  const TopoDS_Wire outer = make_profile_wire(job, profile_index, transform);
  BRepBuilderAPI_MakeFace face(outer, true);
  if (!face.IsDone()) {
    throw std::runtime_error("OCCT could not build a profile face");
  }
  return face.Face();
}

gp_Ax2 profile_fixed_axes(const FfiJob& job, std::size_t profile_index) {
  if (profile_index + 1 >= job.profile_offsets.size()) {
    throw std::runtime_error("profile offset buffer is malformed");
  }
  const std::size_t begin = job.profile_offsets[profile_index];
  const std::size_t end = job.profile_offsets[profile_index + 1];
  if (end < begin + 3) {
    throw std::runtime_error("fixed sweep orientation needs three profile points");
  }
  const gp_Pnt origin = point_at(job, begin);
  gp_Vec x(origin, point_at(job, begin + 1));
  if (x.SquareMagnitude() < 1e-18) {
    throw std::runtime_error("fixed sweep profile axis is degenerate");
  }
  gp_Vec normal;
  bool found_normal = false;
  for (std::size_t index = begin + 2; index < end; ++index) {
    normal = x.Crossed(gp_Vec(origin, point_at(job, index)));
    if (normal.SquareMagnitude() >= 1e-18) {
      found_normal = true;
      break;
    }
  }
  if (!found_normal) {
    throw std::runtime_error("fixed sweep profile plane is degenerate");
  }
  return gp_Ax2(origin, gp_Dir(normal), gp_Dir(x));
}

void configure_pipe(const FfiJob& job, BRepOffsetAPI_MakePipeShell& pipe,
                    std::size_t profile_index, bool allow_guide) {
  if (job.orientation == 0) {
    pipe.SetMode(false);
  } else if (job.orientation == 1) {
    pipe.SetMode(true);
  } else if (job.orientation == 2) {
    pipe.SetMode(profile_fixed_axes(job, profile_index));
  } else {
    throw std::runtime_error("unknown sweep orientation");
  }
  if (job.transition == 0) {
    pipe.SetTransitionMode(BRepBuilderAPI_Transformed);
  } else if (job.transition == 1) {
    pipe.SetTransitionMode(BRepBuilderAPI_RightCorner);
  } else if (job.transition == 2) {
    pipe.SetTransitionMode(BRepBuilderAPI_RoundCorner);
  } else {
    throw std::runtime_error("unknown sweep transition");
  }
  pipe.SetForceApproxC1(job.force_c1);
  if (allow_guide && !job.guide_curve_kinds.empty()) {
    const TopoDS_Wire guide =
        make_curve_wire(job.guide_curve_kinds, job.guide_curve_point_offsets,
                        job.guide_curve_points, "guide rail");
    pipe.SetMode(guide, true, BRepFill_ContactOnBorder);
  }
}

TopoDS_Shape make_exact_face_tool(const FfiJob& job,
                                  const TopoDS_Face& source_face) {
  BRepAdaptor_Surface surface(source_face, true);
  if (surface.GetType() != GeomAbs_Plane) {
    throw std::runtime_error("Extrude source face is not planar");
  }
  gp_Vec direction(job.normal_x, job.normal_y, job.normal_z);
  if (direction.SquareMagnitude() < 1e-18) {
    throw std::runtime_error("extrude normal is degenerate");
  }
  direction.Normalize();

  auto transformed_shape = [&](const TopoDS_Shape& shape, double offset,
                               double scale, const gp_Pnt& center) {
    if (!std::isfinite(scale) || scale <= 1e-6) {
      throw std::runtime_error("taper collapses or inverts the planar face");
    }
    const gp_Vec translation = direction.Multiplied(offset);
    gp_Trsf transform;



    transform.SetValues(
        scale, 0.0, 0.0,
        center.X() * (1.0 - scale) + translation.X(),
        0.0, scale, 0.0,
        center.Y() * (1.0 - scale) + translation.Y(),
        0.0, 0.0, scale,
        center.Z() * (1.0 - scale) + translation.Z());
    BRepBuilderAPI_Transform transformed(shape, transform, true);
    if (!transformed.IsDone() || transformed.Shape().IsNull()) {
      throw std::runtime_error("OCCT could not transform the planar face");
    }
    return transformed.Shape();
  };

  if (std::abs(job.taper_angle_deg) < 1e-12) {
    GProp_GProps properties;
    BRepGProp::SurfaceProperties(source_face, properties);
    const TopoDS_Shape shifted = transformed_shape(
        source_face, job.start_offset, 1.0, properties.CentreOfMass());
    const TopoDS_Face start_face = TopoDS::Face(shifted);
    gp_Vec prism_direction = direction;
    prism_direction.Multiply(job.end_offset - job.start_offset);
    BRepPrimAPI_MakePrism prism(start_face, prism_direction, true, true);
    if (!prism.IsDone() || prism.Shape().IsNull()) {
      throw std::runtime_error("OCCT exact-face prism construction failed");
    }
    return prism.Shape();
  }

  GProp_GProps properties;
  BRepGProp::SurfaceProperties(source_face, properties);
  const gp_Pnt center = properties.CentreOfMass();
  double radius_sum = 0.0;
  std::size_t radius_count = 0;
  for (TopExp_Explorer vertices(source_face, TopAbs_VERTEX); vertices.More();
       vertices.Next()) {
    radius_sum += center.Distance(BRep_Tool::Pnt(TopoDS::Vertex(vertices.Current())));
    ++radius_count;
  }
  if (radius_count == 0) {
    throw std::runtime_error("planar face has no boundary vertices");
  }
  const double reference_radius =
      std::max(radius_sum / static_cast<double>(radius_count), 1e-6);
  const double tangent = std::tan(job.taper_angle_deg * kPi / 180.0);
  const auto scale_at = [&](double offset) {
    return 1.0 + tangent * offset / reference_radius;
  };

  const TopoDS_Wire outer = BRepTools::OuterWire(source_face);
  if (outer.IsNull()) {
    throw std::runtime_error("planar face has no outer boundary wire");
  }
  std::vector<TopoDS_Wire> wires{outer};
  for (TopExp_Explorer explorer(source_face, TopAbs_WIRE); explorer.More();
       explorer.Next()) {
    const TopoDS_Wire wire = TopoDS::Wire(explorer.Current());
    if (!wire.IsSame(outer)) {
      wires.push_back(wire);
    }
  }

  auto loft_wire = [&](const TopoDS_Wire& wire) {
    const TopoDS_Wire first = TopoDS::Wire(transformed_shape(
        wire, job.start_offset, scale_at(job.start_offset), center));
    const TopoDS_Wire last = TopoDS::Wire(transformed_shape(
        wire, job.end_offset, scale_at(job.end_offset), center));
    BRepOffsetAPI_ThruSections loft(true, true, 1e-7);
    loft.CheckCompatibility(true);
    loft.AddWire(first);
    loft.AddWire(last);
    loft.Build(Message_ProgressRange());
    if (!loft.IsDone() || loft.Shape().IsNull()) {
      throw std::runtime_error("OCCT exact-wire tapered loft failed");
    }
    return loft.Shape();
  };
  TopoDS_Shape result = loft_wire(wires.front());
  for (std::size_t index = 1; index < wires.size(); ++index) {
    const TopoDS_Shape hole = loft_wire(wires[index]);
    BRepAlgoAPI_Cut cut(result, hole, Message_ProgressRange());
    if (!cut.IsDone() || cut.Shape().IsNull()) {
      throw std::runtime_error("OCCT could not preserve a tapered face hole");
    }
    result = cut.Shape();
  }
  return result;
}

TopoDS_Shape make_tool(const FfiJob& job, std::size_t region_index) {
  const auto wire_range = region_range(job, region_index);
  const std::size_t wire_begin = wire_range.first;
  const std::size_t wire_end = wire_range.second;
  const std::size_t begin = job.profile_offsets[wire_begin];
  const std::size_t end = job.profile_offsets[wire_begin + 1];
  if (end <= begin + 2 || end * 3 > job.points.size()) {
    throw std::runtime_error("profile offset is out of range");
  }

  if (job.kind == 1) {
    const gp_Vec direction(job.axis_direction_x, job.axis_direction_y,
                           job.axis_direction_z);
    if (direction.SquareMagnitude() < 1e-18) {
      throw std::runtime_error("revolve axis is degenerate");
    }
    const gp_Ax1 axis(
        gp_Pnt(job.axis_origin_x, job.axis_origin_y, job.axis_origin_z),
        gp_Dir(direction));
    auto revolve_wire = [&](std::size_t wire_index) {
      const TopoDS_Face face = make_profile_face(job, wire_index);
      BRepPrimAPI_MakeRevol revolve(face, axis, job.angle_rad, true);
      if (!revolve.IsDone()) {
        throw std::runtime_error("OCCT revolve construction failed");
      }
      return revolve.Shape();
    };
    TopoDS_Shape result = revolve_wire(wire_begin);
    for (std::size_t wire_index = wire_begin + 1; wire_index < wire_end;
         ++wire_index) {
      const TopoDS_Shape cutter = revolve_wire(wire_index);
      BRepAlgoAPI_Cut cut(result, cutter, Message_ProgressRange());
      if (!cut.IsDone() || cut.Shape().IsNull()) {
        throw std::runtime_error("OCCT could not revolve a profile hole");
      }
      result = cut.Shape();
    }
    return result;
  }
  if (job.kind == 2) {
    const TopoDS_Wire path_wire =
        make_curve_wire(job.path_curve_kinds, job.path_curve_point_offsets,
                        job.path_curve_points, "sweep path");
    auto sweep_wire = [&](std::size_t wire_index) {
      const TopoDS_Wire profile = make_profile_wire(job, wire_index);
      BRepOffsetAPI_MakePipeShell pipe(path_wire);
      configure_pipe(job, pipe, wire_index, wire_index == wire_begin);
      pipe.Add(profile, false, false);
      pipe.Build(Message_ProgressRange());
      if (!pipe.IsDone()) {
        throw std::runtime_error("OCCT sweep construction failed");
      }
      if (!pipe.MakeSolid()) {
        throw std::runtime_error("OCCT sweep could not close into a solid");
      }
      GProp_GProps sweep_properties;
      BRepGProp::VolumeProperties(pipe.Shape(), sweep_properties);
      if (!BRepCheck_Analyzer(pipe.Shape(), true, false).IsValid() ||
          !std::isfinite(sweep_properties.Mass()) ||
          std::abs(sweep_properties.Mass()) <= 1e-9) {
        throw std::runtime_error(
            "Sweep did not produce a valid solid. Place the profile across "
            "the path at its start and avoid a self-intersecting sweep.");
      }
      return pipe.Shape();
    };
    TopoDS_Shape result = sweep_wire(wire_begin);
    for (std::size_t wire_index = wire_begin + 1; wire_index < wire_end;
         ++wire_index) {
      const TopoDS_Shape cutter = sweep_wire(wire_index);
      BRepAlgoAPI_Cut cut(result, cutter, Message_ProgressRange());
      if (!cut.IsDone() || cut.Shape().IsNull()) {
        throw std::runtime_error("OCCT could not sweep a profile hole");
      }
      result = cut.Shape();
    }
    return result;
  }
  if (job.kind != 0 && job.kind != 4) {
    throw std::runtime_error("unknown solid job kind");
  }

  gp_Pnt centroid(0.0, 0.0, 0.0);
  for (std::size_t index = begin; index < end; ++index) {
    const gp_Pnt point = point_at(job, index);
    centroid.SetX(centroid.X() + point.X());
    centroid.SetY(centroid.Y() + point.Y());
    centroid.SetZ(centroid.Z() + point.Z());
  }
  const double count = static_cast<double>(end - begin);
  centroid.SetX(centroid.X() / count);
  centroid.SetY(centroid.Y() / count);
  centroid.SetZ(centroid.Z() / count);
  double radius = 0.0;
  for (std::size_t index = begin; index < end; ++index) {
    radius += centroid.Distance(point_at(job, index));
  }
  radius = std::max(radius / count, 1e-6);

  const SectionTransform first_transform =
      section_transform(job, begin, end, job.start_offset, radius);
  const SectionTransform last_transform =
      section_transform(job, begin, end, job.end_offset, radius);
  if (std::abs(job.taper_angle_deg) < 1e-12) {
    gp_Vec direction(job.normal_x, job.normal_y, job.normal_z);
    direction.Normalize();
    direction.Multiply(job.end_offset - job.start_offset);
    auto prism_wire = [&](std::size_t wire_index) {
      const TopoDS_Face face =
          make_profile_face(job, wire_index, &first_transform);
      BRepPrimAPI_MakePrism prism(face, direction, true, true);
      if (!prism.IsDone()) {
        throw std::runtime_error("OCCT prism construction failed");
      }
      return prism.Shape();
    };
    TopoDS_Shape result = prism_wire(wire_begin);
    for (std::size_t wire_index = wire_begin + 1; wire_index < wire_end;
         ++wire_index) {
      const TopoDS_Shape cutter = prism_wire(wire_index);
      BRepAlgoAPI_Cut cut(result, cutter, Message_ProgressRange());
      if (!cut.IsDone() || cut.Shape().IsNull()) {
        throw std::runtime_error("OCCT could not extrude a profile hole");
      }
      result = cut.Shape();
    }
    return result;
  }

  auto loft_wire = [&](std::size_t wire_index) {
    const TopoDS_Wire first_wire =
        make_profile_wire(job, wire_index, &first_transform);
    const TopoDS_Wire last_wire =
        make_profile_wire(job, wire_index, &last_transform);
    BRepOffsetAPI_ThruSections loft(true, true, 1e-7);
    loft.CheckCompatibility(true);
    loft.AddWire(first_wire);
    loft.AddWire(last_wire);
    loft.Build(Message_ProgressRange());
    if (!loft.IsDone()) {
      throw std::runtime_error("OCCT tapered loft construction failed");
    }
    return loft.Shape();
  };
  TopoDS_Shape result = loft_wire(wire_begin);
  for (std::size_t wire_index = wire_begin + 1; wire_index < wire_end;
       ++wire_index) {
    const TopoDS_Shape hole = loft_wire(wire_index);
    BRepAlgoAPI_Cut cut(result, hole, Message_ProgressRange());
    if (!cut.IsDone()) {
      throw std::runtime_error("OCCT could not taper a profile hole");
    }
    result = cut.Shape();
  }
  return result;
}

TopoDS_Shape make_loft_tool(const FfiJob& job) {
  if (job.region_offsets.size() < 3) {
    throw std::runtime_error("Loft needs at least two sections");
  }
  const std::size_t section_count = job.region_offsets.size() - 1;
  const std::size_t wire_count =
      job.region_offsets[1] - job.region_offsets[0];
  for (std::size_t section = 1; section < section_count; ++section) {
    if (job.region_offsets[section + 1] - job.region_offsets[section] !=
        wire_count) {
      throw std::runtime_error(
          "Loft sections must contain the same number of profile holes");
    }
  }
  const bool guided =
      !job.path_curve_kinds.empty() || !job.guide_curve_kinds.empty();
  auto centerline_wire = [&]() {
    if (!job.path_curve_kinds.empty()) {
      return make_curve_wire(job.path_curve_kinds,
                             job.path_curve_point_offsets,
                             job.path_curve_points, "loft centerline");
    }
    std::vector<gp_Pnt> centroids;
    centroids.reserve(section_count);
    for (std::size_t section = 0; section < section_count; ++section) {
      const std::size_t profile_index = job.region_offsets[section];
      const std::size_t begin = job.profile_offsets[profile_index];
      const std::size_t end = job.profile_offsets[profile_index + 1];
      gp_Pnt centroid(0.0, 0.0, 0.0);
      for (std::size_t index = begin; index < end; ++index) {
        const gp_Pnt point = point_at(job, index);
        centroid.SetX(centroid.X() + point.X());
        centroid.SetY(centroid.Y() + point.Y());
        centroid.SetZ(centroid.Z() + point.Z());
      }
      const double count = static_cast<double>(end - begin);
      centroid.SetX(centroid.X() / count);
      centroid.SetY(centroid.Y() / count);
      centroid.SetZ(centroid.Z() / count);
      centroids.push_back(centroid);
    }
    return make_open_wire(centroids);
  };
  auto loft_wire = [&](std::size_t wire_offset) {
    if (guided) {
      const TopoDS_Wire spine = centerline_wire();
      BRepOffsetAPI_MakePipeShell loft(spine);
      loft.SetMode(false);
      loft.SetForceApproxC1(job.continuity >= 1);
      if (wire_offset == 0 && !job.guide_curve_kinds.empty()) {
        const TopoDS_Wire guide =
            make_curve_wire(job.guide_curve_kinds,
                            job.guide_curve_point_offsets,
                            job.guide_curve_points, "loft guide rail");
        loft.SetMode(guide, true, BRepFill_ContactOnBorder);
      }
      for (std::size_t section = 0; section < section_count; ++section) {
        loft.Add(make_profile_wire(
            job, job.region_offsets[section] + wire_offset), false, false);
      }
      loft.Build(Message_ProgressRange());
      if (!loft.IsDone()) {
        throw std::runtime_error("OCCT guided loft construction failed");
      }
      if (!loft.MakeSolid()) {
        throw std::runtime_error("OCCT guided loft could not close into a solid");
      }
      return loft.Shape();
    }
    BRepOffsetAPI_ThruSections loft(true, job.ruled, 1e-7);
    loft.CheckCompatibility(true);
    if (job.continuity == 0) {
      loft.SetContinuity(GeomAbs_C0);
    } else if (job.continuity == 1) {
      loft.SetContinuity(GeomAbs_G1);
    } else if (job.continuity == 2) {
      loft.SetContinuity(GeomAbs_G2);
    } else {
      throw std::runtime_error("unknown loft continuity");
    }
    for (std::size_t section = 0; section < section_count; ++section) {
      loft.AddWire(make_profile_wire(
          job, job.region_offsets[section] + wire_offset));
    }
    loft.Build(Message_ProgressRange());
    if (!loft.IsDone()) {
      throw std::runtime_error("OCCT loft construction failed");
    }
    return loft.Shape();
  };
  TopoDS_Shape result = loft_wire(0);
  for (std::size_t hole = 1; hole < wire_count; ++hole) {
    const TopoDS_Shape cutter = loft_wire(hole);
    BRepAlgoAPI_Cut cut(result, cutter, Message_ProgressRange());
    if (!cut.IsDone()) {
      throw std::runtime_error("OCCT could not loft a profile hole");
    }
    result = cut.Shape();
  }
  return result;
}

TopoDS_Shape fuse_shapes(const std::vector<TopoDS_Shape>& shapes) {
  if (shapes.empty()) {
    throw std::runtime_error("extrude contains no tool profiles");
  }
  TopoDS_Shape result = shapes.front();
  for (std::size_t index = 1; index < shapes.size(); ++index) {
    BRepAlgoAPI_Fuse fuse(result, shapes[index], Message_ProgressRange());
    if (!fuse.IsDone()) {
      throw std::runtime_error("OCCT could not combine tool profiles");
    }
    fuse.SimplifyResult(true, true, 1.0e-7);
    result = fuse.Shape();
  }
  return result;
}

void append_point(rust::Vec<double>& output, const gp_Pnt& point) {
  output.push_back(point.X());
  output.push_back(point.Y());
  output.push_back(point.Z());
}

std::vector<gp_Pnt> sample_projection_edge(const TopoDS_Edge& edge,
                                           double deflection) {
  BRepAdaptor_Curve curve(edge);
  std::vector<gp_Pnt> points;
  if (curve.GetType() == GeomAbs_Line) {
    points.push_back(curve.Value(curve.FirstParameter()));
    points.push_back(curve.Value(curve.LastParameter()));
    return points;
  }
  GCPnts_UniformDeflection discretization(
      curve, std::max(1.0e-4, deflection), true);
  if (discretization.IsDone() && discretization.NbPoints() >= 2) {
    points.reserve(discretization.NbPoints());
    for (int index = 1; index <= discretization.NbPoints(); ++index) {
      points.push_back(discretization.Value(index));
    }
    return points;
  }
  const double first = curve.FirstParameter();
  const double last = curve.LastParameter();
  constexpr int kFallbackSamples = 25;
  points.reserve(kFallbackSamples);
  for (int index = 0; index < kFallbackSamples; ++index) {
    const double parameter =
        first + (last - first) * static_cast<double>(index) /
                    static_cast<double>(kFallbackSamples - 1);
    points.push_back(curve.Value(parameter));
  }
  return points;
}

std::vector<gp_Pnt> sample_section_edge(const TopoDS_Edge& edge,
                                      double deflection, std::size_t limit,
                                      const SectionProgress& progress) {
  progress.check("curve sampling");
  BRepAdaptor_Curve curve(edge);
  std::vector<gp_Pnt> points;
  auto append = [&](const gp_Pnt& point) {
    progress.check("curve sampling");
    if (points.size() >= limit) {
      throw std::runtime_error("Section curve exceeds the native point budget");
    }
    if (!std::isfinite(point.X()) || !std::isfinite(point.Y()) || !std::isfinite(point.Z())) {
      throw std::runtime_error("Section curve contains non-finite coordinates");
    }
    points.push_back(point);
  };
  if (curve.GetType() == GeomAbs_Line) {
    append(curve.Value(curve.FirstParameter()));
    append(curve.Value(curve.LastParameter()));
    return points;
  }
  const int intervals = curve.NbIntervals(GeomAbs_C2);
  if (intervals <= 0 || static_cast<std::size_t>(intervals) >= limit) {
    throw std::runtime_error("Section curve continuity exceeds the native point budget");
  }
  TColStd_Array1OfReal parameters(1, intervals + 1);
  curve.Intervals(parameters, GeomAbs_C2);
  for (int interval = 1; interval <= intervals; ++interval) {
    CPnts_UniformDeflection samples(curve, deflection,
        parameters(interval), parameters(interval + 1), Precision::PConfusion(), true);
    for (; samples.More(); samples.Next()) {
      append(samples.Point());
    }
    if (!samples.IsAllDone()) {
      throw std::runtime_error("OCCT section curve sampling failed");
    }
  }
  return points;
}

std::vector<std::int64_t> projection_polyline_key(
    const std::vector<gp_Pnt>& points) {
  constexpr double kQuantize = 1.0e7;
  std::vector<std::int64_t> forward;
  std::vector<std::int64_t> reverse;
  forward.reserve(points.size() * 2);
  reverse.reserve(points.size() * 2);
  for (const gp_Pnt& point : points) {
    forward.push_back(static_cast<std::int64_t>(std::llround(point.X() * kQuantize)));
    forward.push_back(static_cast<std::int64_t>(std::llround(point.Y() * kQuantize)));
  }
  for (auto iterator = points.rbegin(); iterator != points.rend(); ++iterator) {
    reverse.push_back(static_cast<std::int64_t>(std::llround(iterator->X() * kQuantize)));
    reverse.push_back(static_cast<std::int64_t>(std::llround(iterator->Y() * kQuantize)));
  }
  return reverse < forward ? reverse : forward;
}

void append_projection_shape(
    const TopoDS_Shape& shape,
    double deflection,
    rust::Vec<std::uint32_t>& offsets,
    rust::Vec<double>& coordinates,
    std::set<std::vector<std::int64_t>>& seen) {
  if (shape.IsNull()) {
    return;
  }
  for (TopExp_Explorer explorer(shape, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    const TopoDS_Edge edge = TopoDS::Edge(explorer.Current());
    std::vector<gp_Pnt> points = sample_projection_edge(edge, deflection);
    if (points.size() < 2) {
      continue;
    }
    const auto key = projection_polyline_key(points);
    if (!seen.insert(key).second) {
      continue;
    }
    for (const gp_Pnt& point : points) {
      coordinates.push_back(point.X());
      coordinates.push_back(point.Y());
    }
    offsets.push_back(static_cast<std::uint32_t>(coordinates.size() / 2));
  }
}

void append_section_shape(
    const TopoDS_Shape& shape,
    const gp_Vec& right,
    const gp_Vec& page_up,
    double deflection,
    rust::Vec<std::uint32_t>& offsets,
    rust::Vec<double>& coordinates,
    std::set<std::vector<std::int64_t>>& seen,
    const SectionProgress* progress = nullptr,
    std::size_t point_limit = std::numeric_limits<std::uint32_t>::max()) {
  if (shape.IsNull()) {
    return;
  }
  constexpr double kQuantize = 1.0e7;
  for (TopExp_Explorer explorer(shape, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    const TopoDS_Edge edge = TopoDS::Edge(explorer.Current());
    const std::vector<gp_Pnt> points = progress
        ? sample_section_edge(edge, deflection, point_limit - coordinates.size() / 2, *progress)
        : sample_projection_edge(edge, deflection);
    if (points.size() < 2) {
      continue;
    }
    std::vector<std::int64_t> forward;
    std::vector<std::int64_t> reverse;
    std::vector<std::array<double, 2>> projected;
    projected.reserve(points.size());
    for (const gp_Pnt& point : points) {
      const double x = point.X() * right.X() + point.Y() * right.Y() + point.Z() * right.Z();
      const double y = point.X() * page_up.X() + point.Y() * page_up.Y() + point.Z() * page_up.Z();
      constexpr double kCoordinateLimit =
          static_cast<double>(std::numeric_limits<std::int64_t>::max()) / kQuantize / 2.0;
      if (!std::isfinite(x) || !std::isfinite(y) ||
          std::abs(x) > kCoordinateLimit || std::abs(y) > kCoordinateLimit) {
        throw std::runtime_error("Section coordinates exceed the native quantization range");
      }
      projected.push_back({x, y});
      forward.push_back(static_cast<std::int64_t>(std::llround(x * kQuantize)));
      forward.push_back(static_cast<std::int64_t>(std::llround(y * kQuantize)));
    }
    for (auto iterator = projected.rbegin(); iterator != projected.rend(); ++iterator) {
      reverse.push_back(static_cast<std::int64_t>(std::llround((*iterator)[0] * kQuantize)));
      reverse.push_back(static_cast<std::int64_t>(std::llround((*iterator)[1] * kQuantize)));
    }
    const auto& key = reverse < forward ? reverse : forward;
    if (!seen.insert(key).second) {
      continue;
    }
    for (const auto& point : projected) {
      coordinates.push_back(point[0]);
      coordinates.push_back(point[1]);
    }
    offsets.push_back(static_cast<std::uint32_t>(coordinates.size() / 2));
  }
}

void append_vec(rust::Vec<float>& output, const gp_Vec& value) {
  output.push_back(static_cast<float>(value.X()));
  output.push_back(static_cast<float>(value.Y()));
  output.push_back(static_cast<float>(value.Z()));
}

void append_plane(rust::Vec<double>& output, const TopoDS_Face& face) {
  BRepAdaptor_Surface surface(face, true);
  if (surface.GetType() != GeomAbs_Plane) {
    for (int index = 0; index < 13; ++index) {
      output.push_back(0.0);
    }
    return;
  }
  const gp_Pln plane = surface.Plane();
  const gp_Ax3 axes = plane.Position();
  gp_Dir normal = axes.Direction();
  gp_Dir u = axes.XDirection();
  if (face.Orientation() == TopAbs_REVERSED) {
    normal.Reverse();
  }
  gp_Vec v = gp_Vec(normal).Crossed(gp_Vec(u));
  v.Normalize();
  output.push_back(1.0);
  append_point(output, axes.Location());
  output.push_back(u.X());
  output.push_back(u.Y());
  output.push_back(u.Z());
  output.push_back(v.X());
  output.push_back(v.Y());
  output.push_back(v.Z());
  output.push_back(normal.X());
  output.push_back(normal.Y());
  output.push_back(normal.Z());
}

void append_cylinder(rust::Vec<double>& output, const TopoDS_Face& face) {
  BRepAdaptor_Surface surface(face, true);
  if (surface.GetType() != GeomAbs_Cylinder) {
    for (int index = 0; index < 11; ++index) {
      output.push_back(0.0);
    }
    return;
  }
  const gp_Cylinder cylinder = surface.Cylinder();
  const gp_Ax3 axes = cylinder.Position();
  output.push_back(1.0);
  append_point(output, axes.Location());
  output.push_back(axes.Direction().X());
  output.push_back(axes.Direction().Y());
  output.push_back(axes.Direction().Z());
  output.push_back(axes.XDirection().X());
  output.push_back(axes.XDirection().Y());
  output.push_back(axes.XDirection().Z());
  output.push_back(cylinder.Radius());
}

void append_circle(rust::Vec<double>& output, const TopoDS_Edge& edge,
                   const BRepAdaptor_Curve& curve) {
  if (curve.GetType() != GeomAbs_Circle) {
    for (int index = 0; index < 12; ++index) {
      output.push_back(0.0);
    }
    return;
  }
  const gp_Circ circle = curve.Circle();
  const gp_Ax2 axes = circle.Position();
  const double parameter_span =
      std::abs(curve.LastParameter() - curve.FirstParameter());
  const bool closed = edge.Closed() || std::abs(parameter_span - kTau) <= 1.0e-7;
  output.push_back(1.0);
  append_point(output, axes.Location());
  output.push_back(axes.Direction().X());
  output.push_back(axes.Direction().Y());
  output.push_back(axes.Direction().Z());
  output.push_back(axes.XDirection().X());
  output.push_back(axes.XDirection().Y());
  output.push_back(axes.XDirection().Z());
  output.push_back(circle.Radius());
  output.push_back(closed ? 1.0 : 0.0);
}

struct PlanarFaceSignature {
  bool valid = false;
  gp_Pnt centroid;
  gp_Dir normal;
  double area = 0.0;
  double perimeter = 0.0;
  std::uint32_t wire_count = 0;
  std::uint32_t edge_count = 0;
};

PlanarFaceSignature planar_face_signature(const TopoDS_Face& face) {
  PlanarFaceSignature signature;
  BRepAdaptor_Surface surface(face, true);
  if (surface.GetType() != GeomAbs_Plane) {
    return signature;
  }

  GProp_GProps surface_properties;
  BRepGProp::SurfaceProperties(face, surface_properties, false, false);
  GProp_GProps edge_properties;
  BRepGProp::LinearProperties(face, edge_properties, false, false);
  TopTools_IndexedMapOfShape wires;
  TopTools_IndexedMapOfShape edges;
  TopExp::MapShapes(face, TopAbs_WIRE, wires);
  TopExp::MapShapes(face, TopAbs_EDGE, edges);

  gp_Dir normal = surface.Plane().Position().Direction();
  if (face.Orientation() == TopAbs_REVERSED) {
    normal.Reverse();
  }
  signature.valid = true;
  signature.centroid = surface_properties.CentreOfMass();
  signature.normal = normal;
  signature.area = std::abs(surface_properties.Mass());
  signature.perimeter = std::abs(edge_properties.Mass());
  signature.wire_count = static_cast<std::uint32_t>(wires.Extent());
  signature.edge_count = static_cast<std::uint32_t>(edges.Extent());
  return signature;
}

void append_face_signature(rust::Vec<double>& output, const TopoDS_Face& face) {
  const PlanarFaceSignature signature = planar_face_signature(face);
  if (!signature.valid) {
    for (int index = 0; index < 8; ++index) {
      output.push_back(0.0);
    }
    return;
  }
  output.push_back(1.0);
  append_point(output, signature.centroid);
  output.push_back(signature.area);
  output.push_back(signature.perimeter);
  output.push_back(static_cast<double>(signature.wire_count));
  output.push_back(static_cast<double>(signature.edge_count));
}

bool signature_scalar_matches(double actual, double expected) {
  const double scale = std::max({1.0, std::abs(actual), std::abs(expected)});
  return std::abs(actual - expected) <= scale * 1.0e-6;
}

bool planar_face_signature_matches(
    const PlanarFaceSignature& actual,
    const rust::Vec<double>& expected) {
  if (!actual.valid || expected.size() != 10) {
    return false;
  }
  const gp_Pnt expected_centroid(expected[0], expected[1], expected[2]);
  const gp_Vec expected_normal(expected[3], expected[4], expected[5]);
  if (expected_normal.SquareMagnitude() <= 1.0e-18) {
    return false;
  }
  const double length_scale = std::max(
      {1.0, std::sqrt(std::max(actual.area, 0.0)), actual.perimeter});
  if (actual.centroid.Distance(expected_centroid) > length_scale * 1.0e-6) {
    return false;
  }
  gp_Vec normalized_expected = expected_normal;
  normalized_expected.Normalize();
  if (gp_Vec(actual.normal).Dot(normalized_expected) < 1.0 - 1.0e-7) {
    return false;
  }
  return signature_scalar_matches(actual.area, expected[6]) &&
         signature_scalar_matches(actual.perimeter, expected[7]) &&
         actual.wire_count == static_cast<std::uint32_t>(std::llround(expected[8])) &&
         actual.edge_count == static_cast<std::uint32_t>(std::llround(expected[9]));
}

TopoDS_Face resolve_planar_face_reference(
    const TopoDS_Shape& body,
    const FfiJob& job) {
  if (job.source_face_signature.size() != 10) {
    throw std::runtime_error(
        "Extrude source face has no validated topology signature; reselect it");
  }
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(body, TopAbs_FACE, faces);
  std::vector<TopoDS_Face> matches;
  for (int index = 1; index <= faces.Extent(); ++index) {
    const TopoDS_Face face = TopoDS::Face(faces.FindKey(index));
    if (planar_face_signature_matches(
            planar_face_signature(face), job.source_face_signature)) {
      matches.push_back(face);
    }
  }
  if (matches.empty()) {
    throw std::runtime_error(
        "referenced Extrude source face changed or no longer exists");
  }
  if (matches.size() != 1) {
    throw std::runtime_error(
        "referenced Extrude source face is ambiguous after topology change");
  }
  return matches.front();
}









struct PrismaticCorner {
  gp_Pnt start;
  gp_Vec along;
  gp_Dir into_first;
  gp_Dir into_second;
  double first_width;
  double second_width;
  bool concave;
};

bool point_in_face(const TopoDS_Face& face, const gp_Pnt& point, double tolerance) {
  BRepClass_FaceClassifier classifier(face, point, tolerance);
  const TopAbs_State state = classifier.State();
  return state == TopAbs_IN || state == TopAbs_ON;
}


double wall_width(const TopoDS_Face& face, const gp_Pnt& origin, const gp_Dir& direction,
                  double probe, double tolerance) {
  const auto inside = [&](double distance) {
    return point_in_face(face, origin.Translated(gp_Vec(direction) * distance), tolerance);
  };
  if (inside(probe)) {
    return probe;
  }
  double low = 0.0;
  double high = probe;
  for (int iteration = 0; iteration < 48; ++iteration) {
    const double middle = 0.5 * (low + high);
    if (inside(middle)) {
      low = middle;
    } else {
      high = middle;
    }
  }
  return low;
}

std::optional<PrismaticCorner> prismatic_corner(
    const TopoDS_Edge& edge,
    const TopTools_IndexedDataMapOfShapeListOfShape& edge_faces,
    const TopTools_IndexedDataMapOfShapeListOfShape& vertex_faces,
    double probe) {
  if (BRepAdaptor_Curve(edge).GetType() != GeomAbs_Line || !edge_faces.Contains(edge)) {
    return std::nullopt;
  }
  const TopTools_ListOfShape& adjacent = edge_faces.FindFromKey(edge);
  if (adjacent.Extent() != 2) {
    return std::nullopt;
  }
  TopoDS_Face faces[2];
  gp_Dir normals[2];
  int count = 0;
  for (TopTools_ListIteratorOfListOfShape iterator(adjacent); iterator.More();
       iterator.Next(), ++count) {
    faces[count] = TopoDS::Face(iterator.Value());
    BRepAdaptor_Surface surface(faces[count], true);
    if (surface.GetType() != GeomAbs_Plane) {
      return std::nullopt;
    }
    gp_Dir normal = surface.Plane().Axis().Direction();
    if (faces[count].Orientation() == TopAbs_REVERSED) {
      normal.Reverse();
    }
    normals[count] = normal;
  }
  TopoDS_Vertex first, last;
  TopExp::Vertices(edge, first, last, true);
  if (first.IsNull() || last.IsNull()) {
    return std::nullopt;
  }
  const gp_Pnt start = BRep_Tool::Pnt(first);
  const gp_Vec along(start, BRep_Tool::Pnt(last));
  if (along.SquareMagnitude() < 1.0e-12) {
    return std::nullopt;
  }
  const gp_Dir direction(along);


  for (const TopoDS_Vertex& vertex : {first, last}) {
    if (!vertex_faces.Contains(vertex)) {
      return std::nullopt;
    }
    for (TopTools_ListIteratorOfListOfShape iterator(vertex_faces.FindFromKey(vertex));
         iterator.More(); iterator.Next()) {
      const TopoDS_Face face = TopoDS::Face(iterator.Value());
      if (face.IsSame(faces[0]) || face.IsSame(faces[1])) {
        continue;
      }
      BRepAdaptor_Surface surface(face, true);
      if (surface.GetType() != GeomAbs_Plane ||
          std::abs(surface.Plane().Axis().Direction().Dot(direction)) < 1.0 - 1.0e-6) {
        return std::nullopt;
      }
    }
  }
  const gp_Pnt middle = start.Translated(along * 0.5);
  gp_Dir into[2];
  double widths[2];
  for (int side = 0; side < 2; ++side) {
    const double tolerance = std::max(BRep_Tool::Tolerance(faces[side]), 1.0e-7);
    const double step = std::max(probe * 1.0e-3, tolerance * 10.0);
    const gp_Dir candidate = normals[side].Crossed(direction);
    if (point_in_face(faces[side], middle.Translated(gp_Vec(candidate) * step), tolerance)) {
      into[side] = candidate;
    } else if (point_in_face(faces[side], middle.Translated(gp_Vec(candidate.Reversed()) * step),
                             tolerance)) {
      into[side] = candidate.Reversed();
    } else {
      return std::nullopt;
    }
    widths[side] = wall_width(faces[side], middle, into[side], probe, tolerance);
  }

  const bool concave = into[1].Dot(normals[0]) > 0.0;
  return PrismaticCorner{start, along, into[0], into[1], widths[0], widths[1], concave};
}

std::string format_millimetres(double value) {
  std::ostringstream text;
  text.precision(4);
  text << value;
  return text.str();
}

TopoDS_Shape blend_prismatic_corners(const TopoDS_Shape& shape,
                                     const std::vector<TopoDS_Edge>& edges, double size,
                                     bool chamfer, const char* generic_failure) {
  TopTools_IndexedDataMapOfShapeListOfShape edge_faces;
  TopTools_IndexedDataMapOfShapeListOfShape vertex_faces;
  TopExp::MapShapesAndUniqueAncestors(shape, TopAbs_EDGE, TopAbs_FACE, edge_faces, false);
  TopExp::MapShapesAndUniqueAncestors(shape, TopAbs_VERTEX, TopAbs_FACE, vertex_faces, false);
  const double pi = std::acos(-1.0);
  std::vector<std::pair<TopoDS_Shape, bool>> prisms;
  for (const TopoDS_Edge& edge : edges) {

    const std::optional<PrismaticCorner> corner =
        prismatic_corner(edge, edge_faces, vertex_faces, size * 4.0);
    if (!corner) {
      throw std::runtime_error(generic_failure);
    }
    const double angle = corner->into_first.Angle(corner->into_second);
    if (angle < 1.0e-3 || angle > pi - 1.0e-3) {
      throw std::runtime_error(generic_failure);
    }

    const double reach = chamfer ? size : size / std::tan(angle * 0.5);
    const double narrowest = std::min(corner->first_width, corner->second_width);
    if (reach > narrowest + 1.0e-6) {
      throw std::runtime_error("A " + format_millimetres(size) + " mm " +
                               (chamfer ? "chamfer" : "fillet") + " reaches past the " +
                               format_millimetres(narrowest) +
                               " mm wall beside the selected edge");
    }
    const gp_Pnt first_tangent = corner->start.Translated(gp_Vec(corner->into_first) * reach);
    const gp_Pnt second_tangent = corner->start.Translated(gp_Vec(corner->into_second) * reach);
    BRepBuilderAPI_MakeWire wire;
    wire.Add(BRepBuilderAPI_MakeEdge(corner->start, first_tangent));
    if (chamfer) {
      wire.Add(BRepBuilderAPI_MakeEdge(first_tangent, second_tangent));
    } else {
      gp_Vec bisector = gp_Vec(corner->into_first) + gp_Vec(corner->into_second);
      bisector.Normalize();
      const gp_Pnt centre = corner->start.Translated(bisector * (size / std::sin(angle * 0.5)));
      const gp_Pnt crown = centre.Translated(bisector * -size);
      wire.Add(BRepBuilderAPI_MakeEdge(
          GC_MakeArcOfCircle(first_tangent, crown, second_tangent).Value()));
    }
    wire.Add(BRepBuilderAPI_MakeEdge(second_tangent, corner->start));
    if (!wire.IsDone()) {
      throw std::runtime_error(generic_failure);
    }
    BRepBuilderAPI_MakeFace section(wire.Wire(), true);
    if (!section.IsDone()) {
      throw std::runtime_error(generic_failure);
    }
    BRepPrimAPI_MakePrism prism(section.Face(), corner->along);
    if (!prism.IsDone()) {
      throw std::runtime_error(generic_failure);
    }
    prisms.emplace_back(prism.Shape(), corner->concave);
  }
  TopoDS_Shape result = shape;
  for (const auto& [prism, concave] : prisms) {
    TopTools_ListOfShape arguments;
    arguments.Append(result);
    TopTools_ListOfShape tools;
    tools.Append(prism);
    std::unique_ptr<BRepAlgoAPI_BooleanOperation> operation;
    if (concave) {
      operation = std::make_unique<BRepAlgoAPI_Fuse>();
    } else {
      operation = std::make_unique<BRepAlgoAPI_Cut>();
    }
    operation->SetArguments(arguments);
    operation->SetTools(tools);

    operation->SetFuzzyValue(1.0e-6);
    operation->Build(Message_ProgressRange());
    if (!operation->IsDone() || operation->HasErrors() || operation->Shape().IsNull()) {
      throw std::runtime_error(generic_failure);
    }
    result = operation->Shape();
  }


  ShapeUpgrade_UnifySameDomain unify(result, true, true, false);
  unify.Build();
  result = unify.Shape();
  BRepLib::EncodeRegularity(result);
  int solids = 0;
  for (TopExp_Explorer explorer(result, TopAbs_SOLID); explorer.More(); explorer.Next()) {
    ++solids;
  }
  if (solids != 1 || !BRepCheck_Analyzer(result).IsValid()) {
    throw std::runtime_error(generic_failure);
  }
  return result;
}

}

class Kernel::Impl {
 public:
  std::map<std::uint64_t, TopoDS_Shape> bodies;
  std::set<std::uint64_t> imported_display_bodies;
};

Kernel::Kernel() : impl_(std::make_unique<Impl>()) {}
Kernel::~Kernel() = default;

void Kernel::reset() {
  impl_->bodies.clear();
  impl_->imported_display_bodies.clear();
}

void Kernel::apply_job(const FfiJob& job) {
  // Generated/replaced bodies do not inherit the original STEP display policy.
  for (const auto body_id : job.result_body_ids)
    impl_->imported_display_bodies.erase(body_id);
  if (job.kind == 12) {
    if (job.result_body_ids.size() != 1 || job.step_data.empty()) {
      throw std::runtime_error("STEP import buffers are malformed");
    }
    std::string content;
    content.reserve(job.step_data.size());
    for (const std::uint8_t byte : job.step_data) {
      content.push_back(static_cast<char>(byte));
    }
    std::istringstream stream(content);
    STEPControl_Reader reader;
    if (reader.ReadStream("import.step", stream) != IFSelect_RetDone) {
      throw std::runtime_error("OCCT could not read the STEP stream");
    }
    if (reader.TransferRoots(Message_ProgressRange()) <= 0) {
      throw std::runtime_error("STEP file did not contain transferable shapes");
    }
    const TopoDS_Shape shape = reader.OneShape();
    if (shape.IsNull()) {
      throw std::runtime_error("STEP import produced a null shape");
    }
    impl_->bodies[job.result_body_ids[0]] = shape;
    impl_->imported_display_bodies.insert(job.result_body_ids[0]);
    return;
  }
  if (job.kind == 5 || job.kind == 6) {
    if (job.target_body_ids.size() != 1 || job.edge_indices.empty()) {
      throw std::runtime_error("edge refinement needs one body and at least one edge");
    }
    auto found = impl_->bodies.find(job.target_body_ids[0]);
    if (found == impl_->bodies.end()) {
      throw std::runtime_error("edge refinement target body is missing");
    }
    TopTools_IndexedMapOfShape edge_map;
    TopExp::MapShapes(found->second, TopAbs_EDGE, edge_map);
    std::vector<TopoDS_Edge> selected;
    for (const std::uint32_t index : job.edge_indices) {
      if (index >= static_cast<std::uint32_t>(edge_map.Extent())) {
        throw std::runtime_error(job.kind == 5 ? "referenced fillet edge no longer exists"
                                               : "referenced chamfer edge no longer exists");
      }
      selected.push_back(TopoDS::Edge(edge_map.FindKey(index + 1)));
    }



    if (job.kind == 5) {
      BRepFilletAPI_MakeFillet fillet(found->second);
      for (const TopoDS_Edge& edge : selected) {
        fillet.Add(job.radius, edge);
      }
      fillet.Build(Message_ProgressRange());
      found->second = fillet.IsDone()
                          ? fillet.Shape()
                          : blend_prismatic_corners(
                                found->second, selected, job.radius, false,
                                "OCCT could not build the selected solid fillet");
    } else {
      BRepFilletAPI_MakeChamfer chamfer(found->second);
      for (const TopoDS_Edge& edge : selected) {
        chamfer.Add(job.radius, edge);
      }
      chamfer.Build(Message_ProgressRange());
      found->second = chamfer.IsDone()
                          ? chamfer.Shape()
                          : blend_prismatic_corners(
                                found->second, selected, job.radius, true,
                                "OCCT could not build the selected solid chamfer");
    }
    impl_->imported_display_bodies.erase(job.target_body_ids[0]);
    return;
  }
  if (job.kind == 7) {
    if (job.target_body_ids.size() != 1 || job.diameter <= 0.0 ||
        job.end_offset <= 0.0) {
      throw std::runtime_error("hole parameters are malformed");
    }
    if (job.thread_mode > 0 &&
        (job.thread_nominal_diameter <= job.diameter ||
         job.thread_pitch <= 0.0 || job.thread_depth < 0.0)) {
      throw std::runtime_error("thread parameters are malformed");
    }
    if (job.thread_mode == 2 &&
        (job.thread_major_diameter <= job.thread_pitch_diameter ||
         job.thread_pitch_diameter <= job.thread_minor_diameter ||
         job.thread_minor_diameter <= 0.0)) {
      throw std::runtime_error(
          "modeled thread tolerance limits are malformed");
    }
    auto found = impl_->bodies.find(job.target_body_ids[0]);
    if (found == impl_->bodies.end()) {
      throw std::runtime_error("hole target body is missing");
    }
    gp_Vec direction(job.axis_direction_x, job.axis_direction_y,
                     job.axis_direction_z);
    if (direction.SquareMagnitude() < 1e-18) {
      throw std::runtime_error("hole direction is degenerate");
    }
    direction.Normalize();
    const double overlap = 1e-4;
    const gp_Pnt support(job.axis_origin_x, job.axis_origin_y,
                         job.axis_origin_z);
    const gp_Pnt start = support.Translated(direction.Multiplied(-overlap));
    const gp_Ax2 axis(start, gp_Dir(direction));
    const double hole_depth =
        job.through_all
            ? bounded_through_depth(found->second, job.thread_pitch * 2.0)
            : job.end_offset;


    const double finished_hole_diameter =
        job.thread_mode == 2 ? job.thread_minor_diameter : job.diameter;
    BRepPrimAPI_MakeCylinder main_cylinder(axis, finished_hole_diameter * 0.5,
                                           hole_depth + overlap * 2.0);
    TopoDS_Shape cutter = main_cylinder.Shape();
    std::vector<TopoDS_Shape> thread_cutters;
    if (job.hole_style == 1) {
      BRepPrimAPI_MakeCylinder counterbore(
          axis, job.secondary_diameter * 0.5,
          job.secondary_depth + overlap * 2.0);
      BRepAlgoAPI_Fuse fuse(cutter, counterbore.Shape(),
                            Message_ProgressRange());
      if (!fuse.IsDone()) {
        throw std::runtime_error("OCCT could not build the counterbore cutter");
      }
      cutter = fuse.Shape();
    } else if (job.hole_style == 2) {
      const double large_radius = job.secondary_diameter * 0.5;
      const double small_radius = finished_hole_diameter * 0.5;
      const double half_angle = job.hole_angle_deg * kPi / 360.0;
      const double sink_depth = (large_radius - small_radius) / std::tan(half_angle);
      if (!std::isfinite(sink_depth) || sink_depth <= 0.0) {
        throw std::runtime_error("countersink dimensions are invalid");
      }



      BRepPrimAPI_MakeCone countersink(axis, large_radius + overlap * std::tan(half_angle), small_radius,
                                       sink_depth + overlap);
      BRepAlgoAPI_Fuse fuse(cutter, countersink.Shape(),
                            Message_ProgressRange());
      if (!fuse.IsDone()) {
        throw std::runtime_error("OCCT could not build the countersink cutter");
      }
      cutter = fuse.Shape();
    }
    if (job.thread_mode == 2) {
      const bool full_thread_depth = job.thread_depth <= 0.0;
      const double available_thread_depth =
          job.through_all
              ? bounded_directional_depth(found->second, support, direction)
              : hole_depth;
      const double requested_thread_depth =
          full_thread_depth
              ? available_thread_depth
              : std::min(job.thread_depth, available_thread_depth);


      const gp_Ax2 thread_axis(support, gp_Dir(direction), axis.XDirection());
      thread_cutters = job.thread_form == 1 ? make_rounded_thread_cutters(
          thread_axis, job.thread_major_diameter, job.thread_minor_diameter,
          job.thread_pitch, job.thread_corner_radius, job.thread_axial_clearance,
          requested_thread_depth, job.thread_left_hand, true) : make_internal_thread_cutters(
          thread_axis, job.thread_major_diameter,
          job.thread_pitch_diameter, job.thread_minor_diameter,
          job.thread_pitch, requested_thread_depth, job.thread_left_hand);
      if (!job.through_all ||
          (!full_thread_depth && requested_thread_depth < available_thread_depth - 1e-7)) {
        trim_thread_tools_at_depth(thread_cutters, thread_axis,
            job.thread_major_diameter * 0.5, job.thread_pitch, requested_thread_depth);
      }
    }
    if (!job.through_all && job.hole_bottom_style == 1) {
      const double half_angle = job.drill_point_angle_deg * kPi / 360.0;
      const double tip_depth =
          (finished_hole_diameter * 0.5) / std::tan(half_angle);
      if (!std::isfinite(tip_depth) || tip_depth <= 0.0) {
        throw std::runtime_error("drill point angle is invalid");
      }
      const gp_Pnt tip_start = support.Translated(
          direction.Multiplied(hole_depth - overlap));
      const gp_Ax2 tip_axis(tip_start, gp_Dir(direction));
      BRepPrimAPI_MakeCone drill_point(
          tip_axis, finished_hole_diameter * 0.5, 0.0,
          tip_depth + overlap);
      BRepAlgoAPI_Fuse fuse(cutter, drill_point.Shape(),
                            Message_ProgressRange());
      if (!fuse.IsDone()) {
        throw std::runtime_error("OCCT could not build the drill point cutter");
      }
      cutter = fuse.Shape();
    }
    TopoDS_Shape result;
    if (thread_cutters.empty()) {
      BRepAlgoAPI_Cut cut(found->second, cutter, Message_ProgressRange());
      if (!cut.IsDone() || cut.Shape().IsNull()) {
        throw std::runtime_error("OCCT hole cut failed");
      }
      result = cut.Shape();
    } else if (job.thread_form == 1) {


      BRepAlgoAPI_Cut bore(found->second, cutter, Message_ProgressRange());
      if (!bore.IsDone() || bore.HasErrors() || bore.Shape().IsNull()) {
        throw std::runtime_error("OCCT rounded threaded-hole bore failed");
      }
      result = cut_thread_tools(bore.Shape(), thread_cutters);
    } else {



      result = cut_thread_tools(found->second, thread_cutters);
      BRepAlgoAPI_Cut clean_predrill(
          result, cutter, Message_ProgressRange());
      if (!clean_predrill.IsDone() || clean_predrill.HasErrors() ||
          clean_predrill.Shape().IsNull()) {
        throw std::runtime_error("OCCT threaded-hole predrill cleanup failed");
      }
      result = clean_predrill.Shape();
    }
    if (!thread_cutters.empty()) {
      BRepCheck_Analyzer result_analyzer(result, true, false);
      if (!result_analyzer.IsValid()) {
        throw std::runtime_error("OCCT modeled thread result is invalid");
      }
    }
    found->second = result;
    impl_->imported_display_bodies.erase(job.target_body_ids[0]);
    return;
  }
  if (job.kind == 13) {
    if (job.target_body_ids.size() != 1 || job.face_indices.size() != 1 ||
        job.thread_mode == 0 || job.thread_nominal_diameter <= 0.0 ||
        job.thread_pitch <= 0.0 || job.thread_depth < 0.0) {
      throw std::runtime_error("external thread parameters are malformed");
    }
    if (job.thread_mode == 2 &&
        (job.thread_major_diameter <= job.thread_pitch_diameter ||
         job.thread_pitch_diameter <= job.thread_minor_diameter ||
         job.thread_minor_diameter <= 0.0)) {
      throw std::runtime_error(
          "modeled external thread tolerance limits are malformed");
    }
    auto found = impl_->bodies.find(job.target_body_ids[0]);
    if (found == impl_->bodies.end()) {
      throw std::runtime_error("external thread target body is missing");
    }
    TopTools_IndexedMapOfShape face_map;
    TopExp::MapShapes(found->second, TopAbs_FACE, face_map);
    const std::uint32_t face_index = job.face_indices[0];
    if (face_index >= static_cast<std::uint32_t>(face_map.Extent())) {
      throw std::runtime_error(
          "referenced external-thread cylinder no longer exists");
    }
    const TopoDS_Face face =
        TopoDS::Face(face_map.FindKey(static_cast<int>(face_index) + 1));
    BRepAdaptor_Surface surface(face, true);
    if (surface.GetType() != GeomAbs_Cylinder) {
      throw std::runtime_error(
          "External Thread requires a cylindrical face");
    }
    const gp_Cylinder cylinder = surface.Cylinder();
    const double major_diameter = cylinder.Radius() * 2.0;
    const double diameter_tolerance =
        std::max(0.01, job.thread_nominal_diameter * 0.002);
    if (std::abs(major_diameter - job.thread_nominal_diameter) >
        diameter_tolerance) {
      throw std::runtime_error(
          "selected cylinder does not match the thread major diameter");
    }

    const double first_u = surface.FirstUParameter();
    const double last_u = surface.LastUParameter();
    const double first_v = surface.FirstVParameter();
    const double last_v = surface.LastVParameter();
    if (!std::isfinite(first_u) || !std::isfinite(last_u) ||
        !std::isfinite(first_v) || !std::isfinite(last_v) ||
        std::abs(last_u - first_u) < kTau - 1e-5) {
      throw std::runtime_error(
          "External Thread requires a complete 360-degree cylindrical face");
    }
    const gp_Ax3 cylinder_axes = cylinder.Position();
    const gp_Vec base_axis(cylinder_axes.Direction());
    gp_Pnt sample;
    gp_Vec du;
    gp_Vec dv;
    surface.D1((first_u + last_u) * 0.5, (first_v + last_v) * 0.5,
               sample, du, dv);
    gp_Vec normal = du.Crossed(dv);
    if (face.Orientation() == TopAbs_REVERSED) {
      normal.Reverse();
    }
    gp_Vec radial(cylinder_axes.Location(), sample);
    radial.Subtract(base_axis.Multiplied(radial.Dot(base_axis)));
    if (normal.SquareMagnitude() <= 1e-18 ||
        radial.SquareMagnitude() <= 1e-18 || normal.Dot(radial) <= 0.0) {
      throw std::runtime_error(
          "External Thread requires an outward-facing cylindrical surface");
    }

    const gp_Pnt first_point = surface.Value(first_u, first_v);
    const gp_Pnt last_point = surface.Value(first_u, last_v);
    const double first_offset =
        gp_Vec(cylinder_axes.Location(), first_point).Dot(base_axis);
    const double last_offset =
        gp_Vec(cylinder_axes.Location(), last_point).Dot(base_axis);
    const double lower = std::min(first_offset, last_offset);
    const double upper = std::max(first_offset, last_offset);
    const double available_depth = upper - lower;
    if (!std::isfinite(available_depth) || available_depth <= 1e-7) {
      throw std::runtime_error("external thread cylinder has no axial length");
    }
    const bool full_length = job.thread_depth <= 0.0;
    const double requested_depth =
        full_length ? available_depth : job.thread_depth;
    if (requested_depth > available_depth + 1e-6) {
      throw std::runtime_error(
          "external thread length exceeds the selected cylindrical face");
    }
    gp_Vec direction = base_axis;
    double start_offset = lower;
    if (job.inward) {
      direction.Reverse();
      start_offset = upper;
    }
    const gp_Pnt start = cylinder_axes.Location().Translated(
        base_axis.Multiplied(start_offset));
    const gp_Ax2 thread_axis(
        start, gp_Dir(direction), cylinder_axes.XDirection());
    if (job.thread_mode == 2) {
      TopoDS_Shape result = found->second;
      const double crest_reduction =
          major_diameter - job.thread_major_diameter;
      if (crest_reduction > 1e-7) {



        const double trim_overlap = std::max(1e-4, job.thread_pitch * 1e-4);
        const gp_Pnt trim_start =
            start.Translated(direction.Multiplied(-trim_overlap));
        const gp_Ax2 trim_axis(
            trim_start, gp_Dir(direction), cylinder_axes.XDirection());
        BRepPrimAPI_MakeCylinder outer_trim(
            trim_axis, major_diameter * 0.5 + trim_overlap,
            requested_depth + trim_overlap * 2.0);
        BRepPrimAPI_MakeCylinder inner_keep(
            trim_axis, job.thread_major_diameter * 0.5,
            requested_depth + trim_overlap * 2.0);
        BRepAlgoAPI_Cut sleeve(
            outer_trim.Shape(), inner_keep.Shape(), Message_ProgressRange());
        if (!sleeve.IsDone() || sleeve.HasErrors() ||
            sleeve.Shape().IsNull()) {
          throw std::runtime_error(
              "OCCT could not build the external thread class allowance");
        }
        BRepAlgoAPI_Cut trim(result, sleeve.Shape(), Message_ProgressRange());
        if (!trim.IsDone() || trim.HasErrors() || trim.Shape().IsNull()) {
          throw std::runtime_error(
              "OCCT could not apply the external thread class allowance");
        }
        result = trim.Shape();
      }
      std::vector<TopoDS_Shape> cutters =
          job.thread_form == 1 ? make_rounded_thread_cutters(
              thread_axis, job.thread_major_diameter, job.thread_minor_diameter,
              job.thread_pitch, job.thread_corner_radius, 0.0,
              requested_depth, job.thread_left_hand, false) : make_external_thread_cutters(
              thread_axis, job.thread_major_diameter,
              job.thread_pitch_diameter, job.thread_minor_diameter,
              job.thread_pitch, requested_depth, job.thread_left_hand);
      trim_thread_tools_at_depth(cutters, thread_axis,
          job.thread_major_diameter * 0.5, job.thread_pitch, requested_depth, true);
      GProp_GProps before_thread_properties;
      BRepGProp::VolumeProperties(result, before_thread_properties);
      result = cut_thread_tools(result, cutters);
      BRepCheck_Analyzer result_analyzer(result, true, false);
      if (!result_analyzer.IsValid()) {
        throw std::runtime_error("OCCT modeled external thread result is invalid");
      }
      GProp_GProps result_properties;
      BRepGProp::VolumeProperties(result, result_properties);
      if (!std::isfinite(result_properties.Mass()) ||
          std::abs(result_properties.Mass()) <= 1e-9) {
        throw std::runtime_error(
            "OCCT modeled external thread removed the entire target body");
      }
      const double removed_thread_volume =
          std::abs(before_thread_properties.Mass()) -
          std::abs(result_properties.Mass());
      const double minimum_cut_volume =
          std::max(1e-8, std::abs(before_thread_properties.Mass()) * 1e-8);
      if (!std::isfinite(removed_thread_volume) ||
          removed_thread_volume <= minimum_cut_volume) {
        throw std::runtime_error(
            "OCCT modeled external thread did not remove material");
      }
      found->second = result;
      impl_->imported_display_bodies.erase(job.target_body_ids[0]);
    }
    return;
  }
  if (job.kind == 8) {
    if (job.target_body_ids.size() != 1 || job.face_indices.empty() ||
        !std::isfinite(job.radius) || job.radius <= 0.0) {
      throw std::runtime_error(
          "Shell needs one body, removable faces, and positive thickness");
    }
    auto found = impl_->bodies.find(job.target_body_ids[0]);
    if (found == impl_->bodies.end()) {
      throw std::runtime_error("Shell target body is missing");
    }
    TopTools_IndexedMapOfShape face_map;
    TopExp::MapShapes(found->second, TopAbs_FACE, face_map);
    TopTools_ListOfShape closing_faces;
    for (const std::uint32_t index : job.face_indices) {
      if (index >= static_cast<std::uint32_t>(face_map.Extent())) {
        throw std::runtime_error("referenced Shell face no longer exists");
      }
      closing_faces.Append(face_map.FindKey(index + 1));
    }
    BRepOffsetAPI_MakeThickSolid shell;
    shell.MakeThickSolidByJoin(
        found->second, closing_faces, job.inward ? -job.radius : job.radius,
        1.0e-3, BRepOffset_Skin, false, false, GeomAbs_Arc, true,
        Message_ProgressRange());
    if (!shell.IsDone() || shell.Shape().IsNull()) {
      throw std::runtime_error("OCCT could not build the selected Shell");
    }
    const TopoDS_Shape result = shell.Shape();
    if (!BRepCheck_Analyzer(result, true, false).IsValid()) {
      throw std::runtime_error("OCCT Shell produced invalid geometry; reduce the wall thickness");
    }
    GProp_GProps before_properties, after_properties;
    BRepGProp::VolumeProperties(found->second, before_properties);
    BRepGProp::VolumeProperties(result, after_properties);
    const double before_volume = std::abs(before_properties.Mass());
    const double after_volume = std::abs(after_properties.Mass());
    const double volume_tolerance = std::max(1e-8, before_volume * 1e-8);


    if (!std::isfinite(after_volume) || after_volume <= volume_tolerance ||
        (job.inward && after_volume >= before_volume - volume_tolerance)) {
      throw std::runtime_error("Shell wall thickness leaves no valid hollow body");
    }
    found->second = result;
    impl_->imported_display_bodies.erase(job.target_body_ids[0]);
    return;
  }
  if (job.kind == 9) {
    if (job.target_body_ids.empty() || job.transform_kinds.empty() ||
        job.transform_values.size() != job.transform_kinds.size() * 10 ||
        job.result_body_ids.size() !=
            job.target_body_ids.size() * job.transform_kinds.size()) {
      throw std::runtime_error("body transform buffers are malformed");
    }
    std::size_t output_index = 0;
    for (std::size_t transform_index = 0;
         transform_index < job.transform_kinds.size(); ++transform_index) {
      const std::size_t offset = transform_index * 10;
      gp_Trsf transform;
      if (job.transform_kinds[transform_index] == 0) {
        const gp_Vec normal(job.transform_values[offset + 3],
                            job.transform_values[offset + 4],
                            job.transform_values[offset + 5]);
        if (normal.SquareMagnitude() < 1e-18) {
          throw std::runtime_error("Mirror plane normal is degenerate");
        }
        transform.SetMirror(gp_Ax2(
            gp_Pnt(job.transform_values[offset],
                   job.transform_values[offset + 1],
                   job.transform_values[offset + 2]),
            gp_Dir(normal)));
      } else if (job.transform_kinds[transform_index] == 1) {
        transform.SetTranslation(
            gp_Vec(job.transform_values[offset],
                   job.transform_values[offset + 1],
                   job.transform_values[offset + 2]));
      } else if (job.transform_kinds[transform_index] == 2) {
        const gp_Vec axis(job.transform_values[offset + 3],
                          job.transform_values[offset + 4],
                          job.transform_values[offset + 5]);
        if (axis.SquareMagnitude() < 1e-18) {
          throw std::runtime_error("Circular Pattern axis is degenerate");
        }
        transform.SetRotation(
            gp_Ax1(gp_Pnt(job.transform_values[offset],
                          job.transform_values[offset + 1],
                          job.transform_values[offset + 2]),
                   gp_Dir(axis)),
            job.transform_values[offset + 6]);
      } else if (job.transform_kinds[transform_index] == 3) {
        const double qx = job.transform_values[offset + 3];
        const double qy = job.transform_values[offset + 4];
        const double qz = job.transform_values[offset + 5];
        const double qw = job.transform_values[offset + 6];
        const double magnitude =
            std::sqrt(qx * qx + qy * qy + qz * qz + qw * qw);
        if (!std::isfinite(magnitude) || magnitude <= 1.0e-12) {
          throw std::runtime_error("Move/Copy rotation is degenerate");
        }
        const double x = qx / magnitude;
        const double y = qy / magnitude;
        const double z = qz / magnitude;
        const double w = qw / magnitude;
        const double px = job.transform_values[offset + 7];
        const double py = job.transform_values[offset + 8];
        const double pz = job.transform_values[offset + 9];
        const double tx = job.transform_values[offset];
        const double ty = job.transform_values[offset + 1];
        const double tz = job.transform_values[offset + 2];
        const double r00 = 1.0 - 2.0 * (y * y + z * z);
        const double r01 = 2.0 * (x * y - z * w);
        const double r02 = 2.0 * (x * z + y * w);
        const double r10 = 2.0 * (x * y + z * w);
        const double r11 = 1.0 - 2.0 * (x * x + z * z);
        const double r12 = 2.0 * (y * z - x * w);
        const double r20 = 2.0 * (x * z - y * w);
        const double r21 = 2.0 * (y * z + x * w);
        const double r22 = 1.0 - 2.0 * (x * x + y * y);
        transform.SetValues(
            r00, r01, r02, px + tx - (r00 * px + r01 * py + r02 * pz),
            r10, r11, r12, py + ty - (r10 * px + r11 * py + r12 * pz),
            r20, r21, r22, pz + tz - (r20 * px + r21 * py + r22 * pz));
      } else {
        throw std::runtime_error("unknown body transform kind");
      }
      for (const std::uint64_t source_id : job.target_body_ids) {
        const auto source = impl_->bodies.find(source_id);
        if (source == impl_->bodies.end()) {
          throw std::runtime_error("body transform source is missing");
        }
        BRepBuilderAPI_Transform operation(source->second, transform, true);
        operation.Build(Message_ProgressRange());
        if (!operation.IsDone() || operation.Shape().IsNull()) {
          throw std::runtime_error("OCCT body transform failed");
        }
        impl_->bodies[job.result_body_ids[output_index++]] =
            operation.Shape();
      }
    }
    return;
  }
  if (job.kind == 10) {
    if (job.target_body_ids.size() < 2) {
      throw std::runtime_error("Combine needs a target and at least one tool body");
    }
    const std::uint64_t target_id = job.target_body_ids[0];
    auto target = impl_->bodies.find(target_id);
    if (target == impl_->bodies.end()) {
      throw std::runtime_error("Combine target body is missing");
    }
    TopoDS_Shape result = target->second;
    if (job.operation == 1) {


      TopTools_ListOfShape arguments;
      arguments.Append(result);
      TopTools_ListOfShape tools;
      for (std::size_t index = 1; index < job.target_body_ids.size(); ++index) {
        const auto tool = impl_->bodies.find(job.target_body_ids[index]);
        if (tool == impl_->bodies.end()) {
          throw std::runtime_error("Combine tool body is missing");
        }
        tools.Append(tool->second);
      }
      BRepAlgoAPI_Fuse operation;
      operation.SetArguments(arguments);
      operation.SetTools(tools);
      operation.Build(Message_ProgressRange());
      if (!operation.IsDone() || operation.Shape().IsNull()) {
        throw std::runtime_error("OCCT Combine Join failed");
      }
      operation.SimplifyResult(true, true, 1.0e-7);
      result = operation.Shape();
    }
    for (std::size_t index = 1; index < job.target_body_ids.size(); ++index) {
      if (job.operation == 1) {
        break;
      }
      const auto tool = impl_->bodies.find(job.target_body_ids[index]);
      if (tool == impl_->bodies.end()) {
        throw std::runtime_error("Combine tool body is missing");
      }
      if (job.operation == 2) {
        BRepAlgoAPI_Cut operation(result, tool->second,
                                  Message_ProgressRange());
        if (!operation.IsDone()) {
          throw std::runtime_error("OCCT Combine Cut failed");
        }
        operation.SimplifyResult(true, true, 1.0e-7);
        result = operation.Shape();
      } else if (job.operation == 3) {
        BRepAlgoAPI_Common operation(result, tool->second,
                                     Message_ProgressRange());
        if (!operation.IsDone()) {
          throw std::runtime_error("OCCT Combine Intersect failed");
        }
        operation.SimplifyResult(true, true, 1.0e-7);
        result = operation.Shape();
      } else {
        throw std::runtime_error("unknown Combine operation");
      }
      if (result.IsNull()) {
        throw std::runtime_error("Combine produced a null body");
      }
    }
    impl_->bodies[target_id] = result;
    impl_->imported_display_bodies.erase(target_id);
    if (!job.keep_tools) {
      for (std::size_t index = 1; index < job.target_body_ids.size(); ++index) {
        impl_->bodies.erase(job.target_body_ids[index]);
        impl_->imported_display_bodies.erase(job.target_body_ids[index]);
      }
    }
    return;
  }
  if (job.kind == 11) {
    if (job.target_body_ids.size() != 1 || job.result_body_ids.size() != 2) {
      throw std::runtime_error("Split Body buffers are malformed");
    }
    const auto target = impl_->bodies.find(job.target_body_ids[0]);
    if (target == impl_->bodies.end()) {
      throw std::runtime_error("Split Body target is missing");
    }
    const gp_Vec normal(job.axis_direction_x, job.axis_direction_y,
                        job.axis_direction_z);
    if (normal.SquareMagnitude() < 1e-18) {
      throw std::runtime_error("Split Body plane normal is degenerate");
    }
    const gp_Pnt origin(job.axis_origin_x, job.axis_origin_y,
                        job.axis_origin_z);
    const gp_Vec unit = normal.Normalized();
    BRepBuilderAPI_MakeFace plane(gp_Pln(origin, gp_Dir(unit)));
    if (!plane.IsDone()) {
      throw std::runtime_error("OCCT could not build the splitting plane");
    }
    TopTools_ListOfShape arguments;
    arguments.Append(target->second);
    TopTools_ListOfShape tools;
    tools.Append(plane.Face());
    BRepAlgoAPI_Splitter splitter;
    splitter.SetArguments(arguments);
    splitter.SetTools(tools);
    splitter.SetNonDestructive(true);
    splitter.SetRunParallel(true);
    splitter.Build(Message_ProgressRange());
    if (!splitter.IsDone() || splitter.HasErrors() ||
        splitter.Shape().IsNull()) {
      throw std::runtime_error("OCCT Split Body failed");
    }
    splitter.SimplifyResult(true, true, 1.0e-7);






    struct SplitSolid {
      TopoDS_Shape shape;
      double volume;
      double bounding_volume;
    };
    std::vector<SplitSolid> positive_solids;
    std::vector<SplitSolid> negative_solids;
    for (TopExp_Explorer solids(splitter.Shape(), TopAbs_SOLID); solids.More();
         solids.Next()) {
      const TopoDS_Shape solid = solids.Current();
      GProp_GProps properties;
      BRepGProp::VolumeProperties(solid, properties);
      const double volume = std::abs(properties.Mass());
      const gp_Vec offset(origin, properties.CentreOfMass());
      Bnd_Box solid_bounds;
      BRepBndLib::Add(solid, solid_bounds);
      double sx0 = 0.0;
      double sy0 = 0.0;
      double sz0 = 0.0;
      double sx1 = 0.0;
      double sy1 = 0.0;
      double sz1 = 0.0;
      solid_bounds.Get(sx0, sy0, sz0, sx1, sy1, sz1);
      const double bounding_volume =
          std::max(0.0, sx1 - sx0) * std::max(0.0, sy1 - sy0) *
          std::max(0.0, sz1 - sz0);
      if (!std::isfinite(volume) || volume <= 1e-9) {
        continue;
      }
      const SplitSolid output{solid, volume, bounding_volume};
      if (offset.Dot(unit) >= 0.0) {
        positive_solids.push_back(output);
      } else {
        negative_solids.push_back(output);
      }
    }





    const auto remove_boolean_slivers = [](std::vector<SplitSolid>& solids) {
      if (solids.size() < 2) {
        return;
      }
      const double largest_volume = std::max_element(
          solids.begin(), solids.end(),
          [](const SplitSolid& left, const SplitSolid& right) {
            return left.volume < right.volume;
          })->volume;
      const double relative_limit =
          std::max(1e-8, largest_volume * 1e-5);
      solids.erase(
          std::remove_if(
              solids.begin(), solids.end(),
              [&](const SplitSolid& solid) {
                const double fill_ratio = solid.bounding_volume > 1e-12
                                              ? solid.volume / solid.bounding_volume
                                              : 1.0;
                return solid.volume < relative_limit && fill_ratio < 1e-3;
              }),
          solids.end());
    };
    remove_boolean_slivers(positive_solids);
    remove_boolean_slivers(negative_solids);
    if (positive_solids.empty() || negative_solids.empty()) {
      throw std::runtime_error(
          "Split Body plane does not divide the target into two bodies");
    }
    const auto grouped_shape = [](const std::vector<SplitSolid>& solids) {
      if (solids.size() == 1) {
        return solids.front().shape;
      }
      TopoDS_Compound compound;
      BRep_Builder builder;
      builder.MakeCompound(compound);
      for (const SplitSolid& solid : solids) {
        builder.Add(compound, solid.shape);
      }
      return TopoDS_Shape(compound);
    };
    TopoDS_Shape positive = grouped_shape(positive_solids);
    TopoDS_Shape negative = grouped_shape(negative_solids);
    BRepCheck_Analyzer positive_analyzer(positive, true, false);
    BRepCheck_Analyzer negative_analyzer(negative, true, false);
    if (!positive_analyzer.IsValid() || !negative_analyzer.IsValid()) {





      const auto split_with_halfspace = [&](const gp_Vec& side) {
        const gp_Pnt reference = origin.Translated(side);
        BRepPrimAPI_MakeHalfSpace halfspace(plane.Face(), reference);
        if (!halfspace.IsDone()) {
          throw std::runtime_error(
              "OCCT could not build the Split Body fallback half-space");
        }
        BRepAlgoAPI_Common common(
            target->second, halfspace.Solid(), Message_ProgressRange());
        if (!common.IsDone() || common.HasErrors() ||
            common.Shape().IsNull()) {
          throw std::runtime_error(
              "OCCT Split Body half-space fallback failed");
        }
        common.SimplifyResult(true, true, 1.0e-7);
        TopoDS_Shape result = common.Shape();
        BRepCheck_Analyzer analyzer(result, true, false);
        if (!analyzer.IsValid()) {
          ShapeFix_Shape fixer(result);
          fixer.SetPrecision(1e-7);
          fixer.SetMaxTolerance(1e-5);
          fixer.Perform(Message_ProgressRange());
          result = fixer.Shape();
          BRepLib::SameParameter(result, 1e-6, true);
        }
        return result;
      };
      positive = split_with_halfspace(unit);
      negative = split_with_halfspace(unit.Reversed());
      positive_analyzer = BRepCheck_Analyzer(positive, true, false);
      negative_analyzer = BRepCheck_Analyzer(negative, true, false);
      if (!positive_analyzer.IsValid() || !negative_analyzer.IsValid()) {
        throw std::runtime_error("OCCT Split Body produced invalid geometry");
      }
    }
    impl_->bodies[job.result_body_ids[0]] = positive;
    impl_->bodies[job.result_body_ids[1]] = negative;
    return;
  }
  std::vector<TopoDS_Shape> tools;
  if (job.source_body_id != 0) {
    if (job.kind != 0 || job.source_face_index == UINT32_MAX) {
      throw std::runtime_error("exact face source is only valid for Extrude");
    }
    auto source_body = impl_->bodies.find(job.source_body_id);
    if (source_body == impl_->bodies.end()) {
      throw std::runtime_error("Extrude source body is missing");
    }



    tools.push_back(make_exact_face_tool(
        job, resolve_planar_face_reference(source_body->second, job)));
  } else {
    if (job.profile_offsets.size() < 2 ||
        job.profile_offsets[job.profile_offsets.size() - 1] * 3 !=
            job.points.size()) {
      throw std::runtime_error("profile buffers are malformed");
    }
    if (job.region_offsets.size() < 2 || job.region_offsets.front() != 0 ||
        job.region_offsets.back() + 1 != job.profile_offsets.size()) {
      throw std::runtime_error("profile region buffers are malformed");
    }
    const std::size_t profile_count = job.region_offsets.size() - 1;
    if (job.kind == 3) {
      tools.push_back(make_loft_tool(job));
    } else {
      tools.reserve(profile_count);
      for (std::size_t index = 0; index < profile_count; ++index) {
        tools.push_back(make_tool(job, index));
      }
    }
  }

  if (job.operation == 0) {
    if (job.result_body_ids.size() != tools.size()) {
      throw std::runtime_error("New Body output count does not match profiles");
    }
    for (std::size_t index = 0; index < tools.size(); ++index) {
      impl_->bodies[job.result_body_ids[index]] = tools[index];
    }
    return;
  }
  if (job.operation == 1 && job.target_body_ids.empty()) {
    if (tools.size() < 2 || job.result_body_ids.size() != 1) {
      throw std::runtime_error(
          "Join Profiles needs multiple profiles and one output body");
    }
    impl_->bodies[job.result_body_ids[0]] = fuse_shapes(tools);
    return;
  }
  if (job.target_body_ids.empty()) {
    throw std::runtime_error("boolean solid feature has no target body");
  }
  const TopoDS_Shape tool = fuse_shapes(tools);
  for (const std::uint64_t body_id : job.target_body_ids) {
    auto found = impl_->bodies.find(body_id);
    if (found == impl_->bodies.end()) {
      throw std::runtime_error("boolean target body is missing");
    }
    TopoDS_Shape result;
    if (job.operation == 1) {
      BRepAlgoAPI_Fuse operation(found->second, tool, Message_ProgressRange());
      if (!operation.IsDone()) {
        throw std::runtime_error("OCCT Join failed");
      }
      operation.SimplifyResult(true, true, 1.0e-7);
      result = operation.Shape();
    } else if (job.operation == 2) {
      BRepAlgoAPI_Cut operation(found->second, tool, Message_ProgressRange());
      if (!operation.IsDone()) {
        throw std::runtime_error("OCCT Cut failed");
      }
      operation.SimplifyResult(true, true, 1.0e-7);
      result = operation.Shape();
    } else if (job.operation == 3) {
      BRepAlgoAPI_Common operation(found->second, tool, Message_ProgressRange());
      if (!operation.IsDone()) {
        throw std::runtime_error("OCCT Intersect failed");
      }
      operation.SimplifyResult(true, true, 1.0e-7);
      result = operation.Shape();
    } else {
      throw std::runtime_error("unknown solid operation");
    }
    if (result.IsNull()) {
      throw std::runtime_error("boolean operation produced a null shape");
    }
    found->second = result;
    impl_->imported_display_bodies.erase(body_id);
  }
}

rust::Vec<std::uint64_t> Kernel::body_ids() const {
  rust::Vec<std::uint64_t> result;
  result.reserve(impl_->bodies.size());
  for (const auto& entry : impl_->bodies) {
    result.push_back(entry.first);
  }
  return result;
}




static std::string topology_signature(const TopoDS_Shape& shape) {
  std::uint64_t hash = 14695981039346656037ULL;
  const auto mix = [&hash](std::uint64_t value) {
    for (unsigned int byte = 0; byte < 8; ++byte) {
      hash ^= (value >> (byte * 8)) & 0xffU;
      hash *= 1099511628211ULL;
    }
  };
  TopTools_IndexedMapOfShape vertices, edges, faces;
  TopExp::MapShapes(shape, TopAbs_VERTEX, vertices);
  TopExp::MapShapes(shape, TopAbs_EDGE, edges);
  TopExp::MapShapes(shape, TopAbs_FACE, faces);
  mix(1);
  mix(vertices.Extent()); mix(edges.Extent()); mix(faces.Extent());
  for (int index = 1; index <= edges.Extent(); ++index) {
    const TopoDS_Edge edge = TopoDS::Edge(edges.FindKey(index));
    TopoDS_Vertex first, last;
    TopExp::Vertices(edge, first, last, true);
    mix(BRepAdaptor_Curve(edge).GetType());
    mix(edge.Orientation());
    mix(first.IsNull() ? 0 : vertices.FindIndex(first));
    mix(last.IsNull() ? 0 : vertices.FindIndex(last));
  }
  for (int index = 1; index <= faces.Extent(); ++index) {
    const TopoDS_Face face = TopoDS::Face(faces.FindKey(index));
    mix(BRepAdaptor_Surface(face).GetType());
    mix(face.Orientation());
    for (TopExp_Explorer wire(face, TopAbs_WIRE); wire.More(); wire.Next()) {
      mix(0xf1);
      for (BRepTools_WireExplorer edge(TopoDS::Wire(wire.Current()), face);
           edge.More(); edge.Next()) {
        mix(edges.FindIndex(edge.Current()));
        mix(edge.Current().Orientation());
      }
      mix(0xf2);
    }
    mix(0xf3);
  }
  return std::string("connectivity-v1:") + std::to_string(hash);
}







class TangentBoundaryMeshContext : public BRepMesh_Context {
 public:
  const std::string& BoundaryRepairStop() const { return boundary_repair_stop_; }

  Standard_Boolean DiscretizeFaces(const Message_ProgressRange& range) override {
    const auto& model = GetModel();
    if (model.IsNull()) return false;
    std::set<IMeshData::IFacePtr> eligible;
    for (int fi = 0; fi < model->FacesNb(); ++fi) {
      const auto& face = model->GetFace(fi);
      if (face->GetSurface()->GetType() != GeomAbs_Sphere ||
          (face->GetStatusMask() & ~IMeshData_Outdated) != 0) continue;
      bool clean_wires = true;
      for (int wi = 0; wi < face->WiresNb(); ++wi)
        clean_wires &= face->GetWire(wi)->GetStatusMask() == 0;
      if (clean_wires) eligible.insert(face.get());
    }
    Message_ProgressScope stages(range, "Face triangulation and recovery", 17);
    if (!BRepMesh_Context::DiscretizeFaces(stages.Next())) return false;
    int attempts = 0, successes = 0;
    std::string rejections;
    for (int fi = 0; fi < model->FacesNb() && attempts < 16 && range.More(); ++fi) {
      const auto& face = model->GetFace(fi);
      if (eligible.count(face.get()) == 0 || !face->IsSet(IMeshData_Failure) ||
          (face->GetStatusMask() & ~(IMeshData_Outdated | IMeshData_Failure)) != 0) continue;
      ++attempts;
      if (retry_spherical_face(face, stages.Next())) ++successes;
      else if (attempts - successes <= 2)
        rejections += " face " + std::to_string(fi) + ": " + spherical_retry_stop_.substr(0, 350);
    }
    boundary_repair_stop_ += ", spherical retries/successes " +
        std::to_string(attempts) + '/' + std::to_string(successes);
    if (attempts != successes)
      boundary_repair_stop_ += " rejections" + rejections;
    return true;
  }

  Standard_Boolean HealModel() override {
    const auto& model = GetModel();
    if (model.IsNull()) return false;
    // The default healer can rediscretize edges, so run it before preserving
    // the additional circular-boundary samples below.
    if (!BRepMesh_Context::HealModel()) return false;
    std::set<IMeshData::IFacePtr> affected_faces;
    std::set<IMeshData::IFacePtr> intersection_failures;
    constexpr int unrelated_errors = IMeshData_OpenWire | IMeshData_TooFewPoints |
        IMeshData_UnorientedWire | IMeshData_UserBreak;
    for (int fi = 0; fi < model->FacesNb(); ++fi) {
      const auto& face = model->GetFace(fi);
      if (face->IsSet(IMeshData_SelfIntersectingWire) &&
          (face->GetStatusMask() & unrelated_errors) == 0) {
        bool other_wire_failure = false;
        for (int wi = 0; wi < face->WiresNb(); ++wi) {
          const auto& wire = face->GetWire(wi);
          other_wire_failure |= (wire->GetStatusMask() & unrelated_errors) != 0 ||
              (wire->IsSet(IMeshData_Failure) &&
               !wire->IsSet(IMeshData_SelfIntersectingWire));
        }
        if (!other_wire_failure) intersection_failures.insert(face.get());
      }
    }
    const auto rebuild_pcurves = [&](IMeshData::IEdgePtr edge) {
      struct Endpoints {
        IMeshData::IPCurveHandle pcurve;
        gp_Pnt2d first;
        gp_Pnt2d last;
        double first_parameter;
        double last_parameter;
        TopAbs_Orientation orientation;
      };
      std::vector<Endpoints> endpoints;
      edge->SetStatus(IMeshData_Outdated);
      for (int pi = 0; pi < edge->PCurvesNb(); ++pi) {
        const auto& pcurve = edge->GetPCurve(pi);
        const int count = pcurve->ParametersNb();
        if (count >= 2)
          endpoints.push_back({pcurve, pcurve->GetPoint(0), pcurve->GetPoint(count - 1),
              pcurve->GetParameter(0), pcurve->GetParameter(count - 1),
              pcurve->GetOrientation()});
        pcurve->Clear(false);
        const auto& affected = pcurve->GetFace();
        affected->SetStatus(IMeshData_Outdated);
        affected_faces.insert(affected);
      }
      BRepMesh_EdgeDiscret::Tessellate2d(edge, true);
      // Preserve the healer's connected endpoints only when their parameter
      // and orientation correspondence survived regeneration unchanged.
      for (const auto& saved : endpoints) {
        const int count = saved.pcurve->ParametersNb();
        if (count < 2 || saved.pcurve->GetOrientation() != saved.orientation ||
            saved.pcurve->GetParameter(0) != saved.first_parameter ||
            saved.pcurve->GetParameter(count - 1) != saved.last_parameter)
          continue;
        saved.pcurve->GetPoint(0) = saved.first;
        saved.pcurve->GetPoint(count - 1) = saved.last;
      }
    };
    int junction_attempts = 0, junction_repairs = 0;
    // Collapse only a crossing's contiguous samples inside the shared CAD
    // vertex's tolerance neighborhood. Every adjacent face must still pass.
    const auto repair_junction = [&](const IMeshData::IFaceHandle& face,
                                    const Handle(IMeshData::MapOfIEdgePtr)& crossings) {
      try {
        if (crossings.IsNull() || junction_attempts >= 16 ||
            face->GetSurface()->IsUPeriodic() || face->GetSurface()->IsVPeriodic() ||
            (face->GetStatusMask() & unrelated_errors) != 0 ||
            (face->IsSet(IMeshData_Failure) && intersection_failures.count(face.get()) == 0))
          return false;
        for (int wi = 0; wi < face->WiresNb(); ++wi) {
          const auto& wire = face->GetWire(wi);
          if ((wire->GetStatusMask() & unrelated_errors) != 0 ||
              (wire->IsSet(IMeshData_Failure) && !wire->IsSet(IMeshData_SelfIntersectingWire)))
            return false;
        }
        const auto finite_point = [](const gp_Pnt& point) {
          return std::isfinite(point.X()) && std::isfinite(point.Y()) && std::isfinite(point.Z());
        };
        const auto finite_uv = [](const gp_Pnt2d& point) {
          return std::isfinite(point.X()) && std::isfinite(point.Y());
        };
        const double deflection = GetParameters().Deflection;
        if (!std::isfinite(deflection) || deflection <= 0.0) return false;
        std::size_t comparisons = 0;
        for (int wi = 0; wi < face->WiresNb(); ++wi) {
          const auto& wire = face->GetWire(wi);
          for (int ei = 0; ei < wire->EdgesNb(); ++ei) {
            const int ni = (ei + 1) % wire->EdgesNb();
            auto a = wire->GetEdge(ei), b = wire->GetEdge(ni);
            if (a == b || !crossings->Contains(a) || !crossings->Contains(b) ||
                !a->GetSameParam() || !b->GetSameParam() ||
                !a->GetSameRange() || !b->GetSameRange() ||
                a->PCurvesNb() > 16 || b->PCurvesNb() > 16) continue;
            TopoDS_Vertex vertex;
            if (!TopExp::CommonVertex(a->GetEdge(), b->GetEdge(), vertex)) continue;
            const gp_Pnt center = BRep_Tool::Pnt(vertex);
            const double vertex_tolerance = BRep_Tool::Tolerance(vertex);
            const double a_tolerance = BRep_Tool::Tolerance(a->GetEdge());
            const double b_tolerance = BRep_Tool::Tolerance(b->GetEdge());
            if (!finite_point(center) || !std::isfinite(vertex_tolerance) || vertex_tolerance <= 0.0 ||
                !std::isfinite(a_tolerance) || a_tolerance < 0.0 ||
                !std::isfinite(b_tolerance) || b_tolerance < 0.0) continue;
            const auto& ap = a->GetPCurve(face.get(), wire->GetEdgeOrientation(ei));
            const auto& bp = b->GetPCurve(face.get(), wire->GetEdgeOrientation(ni));
            if (ap.IsNull() || bp.IsNull()) continue;
            const auto endpoint = [&](IMeshData::IEdgePtr edge) {
              const auto& curve = edge->GetCurve();
              const int count = curve->ParametersNb();
              if (count < 2 || count > 4096) return -1;
              TopoDS_Vertex first_vertex, last_vertex;
              TopExp::Vertices(edge->GetEdge(), first_vertex, last_vertex);
              const bool first = !first_vertex.IsNull() && first_vertex.IsSame(vertex) &&
                  curve->GetPoint(0).SquareDistance(center) <= Precision::SquareConfusion();
              const bool last = !last_vertex.IsNull() && last_vertex.IsSame(vertex) &&
                  curve->GetPoint(count - 1).SquareDistance(center) <= Precision::SquareConfusion();
              return first == last ? -1 : first ? 0 : count - 1;
            };
            const int ae = endpoint(a), be = endpoint(b);
            if (ae < 0 || be < 0 ||
                ap->ParametersNb() != a->GetCurve()->ParametersNb() ||
                bp->ParametersNb() != b->GetCurve()->ParametersNb()) continue;
            BRepAdaptor_Curve ac(a->GetEdge()), bc(b->GetEdge());
            BRepAdaptor_Curve af(TopoDS::Edge(a->GetEdge().Oriented(ap->GetOrientation())), face->GetFace());
            BRepAdaptor_Curve bf(TopoDS::Edge(b->GetEdge().Oriented(bp->GetOrientation())), face->GetFace());
            for (int ai = 1; ai < ap->ParametersNb(); ++ai) {
              for (int bi = 1; bi < bp->ParametersNb(); ++bi) {
                if (++comparisons > 16000000) return false;
                const auto& p = ap->GetPoint(ai - 1); const auto& q = ap->GetPoint(ai);
                const auto& r = bp->GetPoint(bi - 1); const auto& s = bp->GetPoint(bi);
                if (!finite_uv(p) || !finite_uv(q) || !finite_uv(r) || !finite_uv(s)) continue;
                if (std::max(p.X(), q.X()) < std::min(r.X(), s.X()) ||
                    std::max(r.X(), s.X()) < std::min(p.X(), q.X()) ||
                    std::max(p.Y(), q.Y()) < std::min(r.Y(), s.Y()) ||
                    std::max(r.Y(), s.Y()) < std::min(p.Y(), q.Y())) continue;
                gp_Pnt2d uv;
                if (BRepMesh_GeomTool::IntSegSeg(p.Coord(), q.Coord(), r.Coord(), s.Coord(),
                        false, false, uv) != BRepMesh_GeomTool::Cross) continue;
                if (!finite_uv(uv)) continue;
                const gp_XY av = q.Coord() - p.Coord(), bv = s.Coord() - r.Coord();
                if (!std::isfinite(av.SquareModulus()) || !std::isfinite(bv.SquareModulus()) ||
                    av.SquareModulus() <= 0.0 || bv.SquareModulus() <= 0.0) continue;
                const double at = ap->GetParameter(ai - 1) +
                    (uv.Coord() - p.Coord()).Dot(av) / av.SquareModulus() *
                        (ap->GetParameter(ai) - ap->GetParameter(ai - 1));
                const double bt = bp->GetParameter(bi - 1) +
                    (uv.Coord() - r.Coord()).Dot(bv) / bv.SquareModulus() *
                        (bp->GetParameter(bi) - bp->GetParameter(bi - 1));
                if (!std::isfinite(at) || !std::isfinite(bt)) continue;
                const auto auv = af.CurveOnSurface().GetCurve()->Value(at);
                const auto buv = bf.CurveOnSurface().GetCurve()->Value(bt);
                if (!finite_uv(auv) || !finite_uv(buv)) continue;
                const double aet = ap->GetParameter(ae), bet = bp->GetParameter(be);
                if (!std::isfinite(aet) || !std::isfinite(bet)) continue;
                const auto aeuv = af.CurveOnSurface().GetCurve()->Value(aet);
                const auto beuv = bf.CurveOnSurface().GetCurve()->Value(bet);
                if (!finite_uv(aeuv) || !finite_uv(beuv)) continue;
                const double ad = face->GetSurface()->Value(auv.X(), auv.Y()).Distance(ac.Value(at));
                const double bd = face->GetSurface()->Value(buv.X(), buv.Y()).Distance(bc.Value(bt));
                const double aed = face->GetSurface()->Value(aeuv.X(), aeuv.Y()).Distance(ac.Value(aet));
                const double bed = face->GetSurface()->Value(beuv.X(), beuv.Y()).Distance(bc.Value(bet));
                const double crossing_distance = face->GetSurface()->Value(uv.X(), uv.Y()).Distance(center);
                // Include the measured source representation error at this
                // junction, always bounded by its edge's existing tolerance.
                if (!std::isfinite(ad) || !std::isfinite(bd) ||
                    !std::isfinite(aed) || !std::isfinite(bed) ||
                    !std::isfinite(crossing_distance) || ad > a_tolerance || bd > b_tolerance ||
                    aed > a_tolerance || bed > b_tolerance ||
                    crossing_distance >
                        vertex_tolerance + std::max({ad, bd, aed, bed, Precision::Confusion()})) continue;
                const auto near_vertex_path = [&](IMeshData::IEdgePtr edge, int end, int segment) {
                  const auto& curve = edge->GetCurve();
                  const int first = end == 0 ? 1 : segment;
                  const int last = end == 0 ? segment - 1 : curve->ParametersNb() - 2;
                  const double limit = vertex_tolerance + BRep_Tool::Tolerance(edge->GetEdge());
                  const gp_Pnt retained = curve->GetPoint(end == 0 ? segment : segment - 1);
                  if (!finite_point(retained)) return false;
                  const gp_Vec chord(center, retained);
                  const double length_squared = chord.SquareMagnitude();
                  if (!std::isfinite(length_squared)) return false;
                  for (int index = first; index <= last; ++index) {
                    const auto& point = curve->GetPoint(index);
                    if (!finite_point(point) || !std::isfinite(curve->GetParameter(index))) return false;
                    const double distance = point.Distance(center);
                    if (!std::isfinite(distance) || distance > limit) return false;
                    const double fraction = length_squared > 0.0
                        ? std::clamp(gp_Vec(center, point).Dot(chord) / length_squared, 0.0, 1.0) : 0.0;
                    const double error = point.Distance(center.Translated(chord.Multiplied(fraction)));
                    if (!std::isfinite(error) || error > deflection) return false;
                  }
                  return true;
                };
                if (!near_vertex_path(a, ae, ai) || !near_vertex_path(b, be, bi)) continue;
                if (junction_attempts >= 16) return false;
                struct PCurveState {
                  IMeshData::IPCurveHandle curve;
                  std::vector<gp_Pnt2d> points;
                  std::vector<double> parameters;
                  std::vector<int> indices;
                };
                struct EdgeState {
                  IMeshData::IEdgePtr edge;
                  int status;
                  std::vector<gp_Pnt> points;
                  std::vector<double> parameters;
                  std::vector<PCurveState> pcurves;
                };
                struct FaceState { IMeshData::IFacePtr face; int status; std::vector<int> wires; };
                std::vector<EdgeState> saved_edges;
                std::map<IMeshData::IFacePtr, FaceState> saved_faces;
                for (auto edge : {a, b}) {
                  EdgeState saved{edge, edge->GetStatusMask(), {}, {}, {}};
                  const auto& curve = edge->GetCurve();
                  for (int index = 0; index < curve->ParametersNb(); ++index) {
                    saved.points.push_back(curve->GetPoint(index));
                    saved.parameters.push_back(curve->GetParameter(index));
                  }
                  for (int pi = 0; pi < edge->PCurvesNb(); ++pi) {
                    const auto& pc = edge->GetPCurve(pi);
                    PCurveState saved_pc{pc, {}, {}, {}};
                    for (int index = 0; index < pc->ParametersNb(); ++index) {
                      saved_pc.points.push_back(pc->GetPoint(index));
                      saved_pc.parameters.push_back(pc->GetParameter(index));
                      saved_pc.indices.push_back(pc->GetIndex(index));
                    }
                    saved.pcurves.push_back(std::move(saved_pc));
                    auto* adjacent = pc->GetFace();
                    if (saved_faces.count(adjacent) == 0) {
                      FaceState state{adjacent, adjacent->GetStatusMask(), {}};
                      for (int index = 0; index < adjacent->WiresNb(); ++index)
                        state.wires.push_back(adjacent->GetWire(index)->GetStatusMask());
                      saved_faces.emplace(adjacent, std::move(state));
                    }
                  }
                  saved_edges.push_back(std::move(saved));
                }
                const auto old_affected_faces = affected_faces;
                const auto restore_status = [](auto* item, int status) {
                  item->UnsetStatus(static_cast<IMeshData_Status>(item->GetStatusMask()));
                  item->SetStatus(static_cast<IMeshData_Status>(status));
                };
                const auto rollback = [&]() {
                  for (const auto& saved : saved_edges) {
                    const auto& curve = saved.edge->GetCurve();
                    curve->Clear(false);
                    for (std::size_t index = 0; index < saved.points.size(); ++index)
                      curve->AddPoint(saved.points[index], saved.parameters[index]);
                    for (const auto& pc : saved.pcurves) {
                      pc.curve->Clear(false);
                      for (std::size_t index = 0; index < pc.points.size(); ++index) {
                        pc.curve->AddPoint(pc.points[index], pc.parameters[index]);
                        pc.curve->GetIndex(static_cast<int>(index)) = pc.indices[index];
                      }
                    }
                    restore_status(saved.edge, saved.status);
                  }
                  for (const auto& entry : saved_faces) {
                    restore_status(entry.first, entry.second.status);
                    for (std::size_t index = 0; index < entry.second.wires.size(); ++index)
                      restore_status(entry.first->GetWire(static_cast<int>(index)).get(), entry.second.wires[index]);
                  }
                  affected_faces = old_affected_faces;
                };
                ++junction_attempts;
                try {
                  for (int which = 0; which < 2; ++which) {
                    const auto& saved = saved_edges[which];
                    const int end = which == 0 ? ae : be, segment = which == 0 ? ai : bi;
                    const auto& curve = saved.edge->GetCurve();
                    curve->Clear(false);
                    for (int index = 0; index < static_cast<int>(saved.points.size()); ++index) {
                      if (index > 0 && index < static_cast<int>(saved.points.size()) - 1 &&
                          (end == 0 ? index < segment : index >= segment)) continue;
                      curve->AddPoint(saved.points[index], saved.parameters[index]);
                    }
                    rebuild_pcurves(saved.edge);
                  }
                  ap->GetPoint(ae == 0 ? 0 : ap->ParametersNb() - 1) = uv;
                  bp->GetPoint(be == 0 ? 0 : bp->ParametersNb() - 1) = uv;
                  bool valid = true;
                  for (const auto& entry : saved_faces) {
                    BRepMesh_FaceChecker after(IMeshData::IFaceHandle(entry.first), GetParameters());
                    if (!after.Perform()) { valid = false; break; }
                  }
                  if (valid) { ++junction_repairs; return true; }
                } catch (const Standard_Failure&) {
                  rollback();
                  continue;
                } catch (const std::exception&) {
                  rollback();
                  continue;
                }
                rollback();
              }
            }
          }
        }
        return false;
      } catch (const Standard_Failure&) {
        return false;
      } catch (const std::exception&) {
        return false;
      }
    };
    const auto check_repaired_faces = [&]() {
      constexpr int max_boundary_passes = 8;
      constexpr int max_edge_points = 4096;
      constexpr std::size_t max_added_points = 65536;
      std::size_t added_points = 0;
      for (int pass = 0; pass <= max_boundary_passes; ++pass) {
        std::set<IMeshData::IEdgePtr> intersecting_edges;
        for (int fi = 0; fi < model->FacesNb(); ++fi) {
          const auto& face = model->GetFace(fi);
          if (affected_faces.count(face.get()) == 0 &&
              !face->IsSet(IMeshData_SelfIntersectingWire)) continue;
          BRepMesh_FaceChecker checker(face, GetParameters());
          bool valid = checker.Perform();
          if (!valid && repair_junction(face, checker.GetIntersectingEdges()))
            valid = checker.Perform();
          if (!valid) {
            if (!face->IsSet(IMeshData_Failure) &&
                (face->GetStatusMask() & unrelated_errors) == 0)
              intersection_failures.insert(face.get());
            face->SetStatus(IMeshData_SelfIntersectingWire);
            face->SetStatus(IMeshData_Failure);
            const auto& edges = checker.GetIntersectingEdges();
            if (!edges.IsNull()) {
              for (IMeshData::MapOfIEdgePtr::Iterator edge(*edges);
                   edge.More(); edge.Next())
                intersecting_edges.insert(edge.Value());
            }
            continue;
          }
          face->UnsetStatus(IMeshData_SelfIntersectingWire);
          if (intersection_failures.count(face.get()) != 0)
            face->UnsetStatus(IMeshData_Failure);
          for (int wi = 0; wi < face->WiresNb(); ++wi) {
            const auto& wire = face->GetWire(wi);
            if (!wire->IsSet(IMeshData_SelfIntersectingWire)) continue;
            wire->UnsetStatus(IMeshData_SelfIntersectingWire);
            if ((wire->GetStatusMask() & unrelated_errors) == 0)
              wire->UnsetStatus(IMeshData_Failure);
          }
        }
        if (intersecting_edges.empty() || pass == max_boundary_passes) {
          boundary_repair_stop_ = intersecting_edges.empty()
              ? "no reported boundary intersections"
              : "8 pass limit";
          boundary_repair_stop_ += ", added points " + std::to_string(added_points);
          boundary_repair_stop_ += ", junction attempts/repairs " +
              std::to_string(junction_attempts) + '/' + std::to_string(junction_repairs);
          break;
        }
        bool inserted = false;
        bool parameter_mismatch = false, edge_limit = false, point_limit = false;
        for (auto edge : intersecting_edges) {
          if (!edge->GetSameParam() || !edge->GetSameRange()) {
            parameter_mismatch = true;
            continue;
          }
          const auto& points = edge->GetCurve();
          const int count = points->ParametersNb();
          if (count < 2) continue;
          if (count > (max_edge_points + 1) / 2) {
            edge_limit = true;
            continue;
          }
          if (static_cast<std::size_t>(count - 1) > max_added_points - added_points) {
            point_limit = true;
            continue;
          }
          BRepAdaptor_Curve curve(edge->GetEdge());
          bool edge_inserted = false;
          // Refine exact curve samples without replacing the samples already
          // needed by adjacent faces or the circular-boundary repair.
          for (int index = count - 1; index > 0; --index) {
            const double first = points->GetParameter(index - 1);
            const double last = points->GetParameter(index);
            const double middle = first + (last - first) * 0.5;
            if (!std::isfinite(middle) || middle == first || middle == last)
              continue;
            points->InsertPoint(index, curve.Value(middle), middle);
            ++added_points;
            inserted = true;
            edge_inserted = true;
          }
          if (!edge_inserted) continue;
          rebuild_pcurves(edge);
        }
        if (!inserted) {
          boundary_repair_stop_ = edge_limit ? "4096 edge point limit"
              : point_limit ? "65536 added point limit"
              : parameter_mismatch ? "nonmatching edge parameters"
              : "no representable midpoint";
          boundary_repair_stop_ += ", added points " + std::to_string(added_points);
          boundary_repair_stop_ += ", junction attempts/repairs " +
              std::to_string(junction_attempts) + '/' + std::to_string(junction_repairs);
          break;
        }
      }
      return Standard_True;
    };

    constexpr int max_refinement_passes = 16;
    for (int pass = 0; pass <= max_refinement_passes; ++pass) {
      std::map<IMeshData::IEdgePtr, std::vector<double>> additions;
      bool crossing = false;
      std::string crossing_detail;
      for (int fi = 0; fi < model->FacesNb(); ++fi) {
        const auto& face = model->GetFace(fi);
        if (face->GetSurface()->GetType() != GeomAbs_Plane) continue;
        for (int wi = 0; wi < face->WiresNb(); ++wi) {
          const auto& wire = face->GetWire(wi);
          for (int ei = 0; ei < wire->EdgesNb(); ++ei) {
            const int ni = (ei + 1) % wire->EdgesNb();
            auto a = wire->GetEdge(ei);
            auto b = wire->GetEdge(ni);
            if (a == b || !a->GetSameParam() || !b->GetSameParam() ||
                !a->GetSameRange() || !b->GetSameRange()) continue;
            BRepAdaptor_Curve ac(a->GetEdge()), bc(b->GetEdge());
            const bool a_circle = ac.GetType() == GeomAbs_Circle;
            if (a_circle == (bc.GetType() == GeomAbs_Circle)) continue;
            const auto& ap = a->GetPCurve(face.get(), wire->GetEdgeOrientation(ei));
            const auto& bp = b->GetPCurve(face.get(), wire->GetEdgeOrientation(ni));
            TopoDS_Vertex shared_vertex;
            const bool has_shared_vertex =
                TopExp::CommonVertex(a->GetEdge(), b->GetEdge(), shared_vertex);
            const double junction_tolerance = has_shared_vertex
                ? std::max({Precision::Confusion(),
                            BRep_Tool::Tolerance(shared_vertex),
                            BRep_Tool::Tolerance(a->GetEdge()),
                            BRep_Tool::Tolerance(b->GetEdge())})
                : Precision::Confusion();
            auto cross = [](const gp_Pnt2d& p, const gp_Pnt2d& q, const gp_Pnt2d& r) {
              return (q.X()-p.X())*(r.Y()-p.Y()) - (q.Y()-p.Y())*(r.X()-p.X());
            };
            for (int ai = 1; ai < ap->ParametersNb(); ++ai) {
              for (int bi = 1; bi < bp->ParametersNb(); ++bi) {
                const auto& p = ap->GetPoint(ai-1); const auto& q = ap->GetPoint(ai);
                const auto& r = bp->GetPoint(bi-1); const auto& s = bp->GetPoint(bi);



                if (std::min({p.SquareDistance(r), p.SquareDistance(s),
                              q.SquareDistance(r), q.SquareDistance(s)}) <=
                    Precision::SquareConfusion()) continue;
                if (cross(p,q,r)*cross(p,q,s) >= -1e-18 ||
                    cross(r,s,p)*cross(r,s,q) >= -1e-18) continue;
                const double denominator =
                    (q.X()-p.X())*(s.Y()-r.Y()) -
                    (q.Y()-p.Y())*(s.X()-r.X());
                const double fraction =
                    ((r.X()-p.X())*(s.Y()-r.Y()) -
                     (r.Y()-p.Y())*(s.X()-r.X())) / denominator;
                const gp_Pnt2d intersection(
                    p.X() + fraction*(q.X()-p.X()),
                    p.Y() + fraction*(q.Y()-p.Y()));
                const double junction_distance = has_shared_vertex
                    ? face->GetSurface()->Value(intersection.X(), intersection.Y())
                          .Distance(BRep_Tool::Pnt(shared_vertex))
                    : std::numeric_limits<double>::infinity();
                // STEP vertices can tolerate a junction wider than Confusion.
                // Refining a crossing inside that junction cannot repair the
                // exact curves and can exhaust every pass on valid geometry.
                if (has_shared_vertex && junction_distance <= junction_tolerance)
                    continue;
                if (crossing_detail.empty()) {
                  const auto curve_name = [](GeomAbs_CurveType type) {
                    switch (type) {
                      case GeomAbs_Line: return "line";
                      case GeomAbs_Circle: return "circle";
                      case GeomAbs_Ellipse: return "ellipse";
                      case GeomAbs_Hyperbola: return "hyperbola";
                      case GeomAbs_Parabola: return "parabola";
                      case GeomAbs_BezierCurve: return "Bezier";
                      case GeomAbs_BSplineCurve: return "B-spline";
                      case GeomAbs_OffsetCurve: return "offset";
                      default: return "other";
                    }
                  };
                  std::ostringstream detail;
                  detail.precision(17);
                  detail << " (face " << fi << ", wire " << wi
                         << ", edges " << ei << '/' << ni
                         << ", curves " << curve_name(ac.GetType()) << '/'
                         << curve_name(bc.GetType())
                         << ", parameters " << ap->GetParameter(ai-1) << ':'
                         << ap->GetParameter(ai) << '/' << bp->GetParameter(bi-1)
                         << ':' << bp->GetParameter(bi)
                         << ", junction distance mm " << junction_distance
                         << ", junction tolerance mm " << junction_tolerance
                         << ", pass " << pass << ')';
                  crossing_detail = detail.str();
                }
                crossing = true;
                auto circular = a_circle ? a : b;
                auto other = a_circle ? b : a;
                const int oi = a_circle ? bi : ai;
                BRepAdaptor_Curve curve(circular->GetEdge());
                BRepAdaptor_Curve other_curve(other->GetEdge());
                const auto& other_pcurve = a_circle ? bp : ap;
                const double first = curve.FirstParameter(), last = curve.LastParameter();
                const double middle = (other_pcurve->GetParameter(oi-1) + other_pcurve->GetParameter(oi)) * 0.5;
                additions[other].push_back(middle);
                for (double sample : {other_pcurve->GetParameter(oi-1), middle, other_pcurve->GetParameter(oi)}) {


                  const gp_Pnt point = other_curve.Value(sample);
                  double parameter = ElCLib::Parameter(curve.Circle(), point);
                  parameter += kTau * std::ceil((first - parameter) / kTau);
                  if (parameter > first+1e-10 && parameter < last-1e-10)
                    additions[circular].push_back(parameter);
                }
              }
            }
          }
        }
      }



      if (!crossing) return check_repaired_faces();
      if (additions.empty() || pass == max_refinement_passes) {
        throw std::runtime_error("OCCT could not discretize tangential face boundaries without crossing chords" + crossing_detail);
      }
      bool inserted = false;
      for (auto& entry : additions) {
        auto edge = entry.first;
        auto& parameters = entry.second;
        std::sort(parameters.begin(), parameters.end());
        BRepAdaptor_Curve curve(edge->GetEdge());
        const auto& points = edge->GetCurve();
        const bool ascending = points->GetParameter(0) <
            points->GetParameter(points->ParametersNb()-1);
        bool edge_inserted = false;
        for (double parameter : parameters) {
          int index = 0;
          while (index < points->ParametersNb() &&
                 (ascending ? points->GetParameter(index) < parameter :
                              points->GetParameter(index) > parameter)) ++index;
          if ((index < points->ParametersNb() && std::abs(points->GetParameter(index)-parameter) < 1e-10) ||
              (index > 0 && std::abs(points->GetParameter(index-1)-parameter) < 1e-10)) continue;
          points->InsertPoint(index, curve.Value(parameter), parameter);
          inserted = true;
          edge_inserted = true;
        }
        if (!edge_inserted) continue;
        rebuild_pcurves(edge);
      }
      if (!inserted) {
        throw std::runtime_error("OCCT could not refine crossing tangential face boundaries" + crossing_detail);
      }
    }
    return true;
  }

 private:
  bool retry_spherical_face(const IMeshData::IFaceHandle& face,
                            const Message_ProgressRange& range) {
    spherical_retry_stop_ = "eligibility or boundary check";
    struct RollbackFailure : std::runtime_error {
      RollbackFailure() : std::runtime_error("OCCT could not restore a failed spherical mesh retry") {}
    };
    try {
      TopLoc_Location old_location;
      if (!BRep_Tool::Triangulation(face->GetFace(), old_location).IsNull()) return false;
      BRepMesh_FaceChecker checker(face, GetParameters());
      if (!checker.Perform()) return false;
      struct Boundary {
        IMeshData::IPCurveHandle pcurve;
        IMeshData::ICurveHandle curve;
        std::vector<int> indices;
      };
      std::vector<Boundary> boundaries;
      std::vector<IMeshData::IWireHandle> wires;
      double expected_uv_area = 0.0;
      int count = 0;
      for (int wi = 0; wi < face->WiresNb(); ++wi) {
        const auto& wire = face->GetWire(wi);
        if (wire->GetStatusMask() != 0) return false;
        wires.push_back(wire);
        std::vector<gp_Pnt2d> polygon;
        for (int ei = 0; ei < wire->EdgesNb(); ++ei) {
          const auto& edge = wire->GetEdge(ei);
          const auto& pc = edge->GetPCurve(face.get(), wire->GetEdgeOrientation(ei));
          const auto& curve = edge->GetCurve();
          if (pc.IsNull() || pc->ParametersNb() < 2 ||
              pc->ParametersNb() != curve->ParametersNb() ||
              (count += pc->ParametersNb()) > 4096 || boundaries.size() >= 16) return false;
          Boundary boundary{pc, curve, {}};
          for (int i = 0; i < pc->ParametersNb(); ++i)
            boundary.indices.push_back(pc->GetIndex(i));
          boundaries.push_back(std::move(boundary));
          for (int i = 0; i < pc->ParametersNb() - 1; ++i)
            polygon.push_back(pc->GetPoint(wire->GetEdgeOrientation(ei) == TopAbs_REVERSED ?
                pc->ParametersNb() - 1 - i : i));
        }
        if (polygon.size() < 3) return false;
        const auto origin = polygon.front().Coord();
        for (std::size_t i = 0; i < polygon.size(); ++i)
          expected_uv_area += 0.5 * (polygon[i].Coord() - origin).Crossed(
              polygon[(i + 1) % polygon.size()].Coord() - origin);
      }
      if (!std::isfinite(expected_uv_area) || expected_uv_area == 0.0) return false;
      const int old_status = face->GetStatusMask();
      const auto rollback = [&]() {
        face->UnsetStatus(static_cast<IMeshData_Status>(face->GetStatusMask()));
        face->SetStatus(static_cast<IMeshData_Status>(old_status));
        for (const auto& wire : wires)
          wire->UnsetStatus(static_cast<IMeshData_Status>(wire->GetStatusMask()));
        for (const auto& boundary : boundaries)
          for (int i = 0; i < static_cast<int>(boundary.indices.size()); ++i)
            boundary.pcurve->GetIndex(i) = boundary.indices[i];
        try {
          BRep_Builder().UpdateFace(face->GetFace(), Handle(Poly_Triangulation)());
        } catch (const Standard_Failure&) {
          throw RollbackFailure();
        } catch (const std::exception&) {
          throw RollbackFailure();
        }
      };
      try {
        // Retry only the failed face, before ModelPostProcessor attaches shared
        // edge polygons. Both factories consume the identical discrete boundary.
        BRepMesh_DelabellaMeshAlgoFactory factory;
        const auto algorithm = factory.GetAlgo(GeomAbs_Sphere, GetParameters());
        if (algorithm.IsNull()) return false;
        // This flag belongs solely to the failed first attempt. Restore it on
        // every rejected trial; success still requires the full validation below.
        face->UnsetStatus(IMeshData_Failure);
        algorithm->Perform(face, GetParameters(), range);
        TopLoc_Location location;
        const auto triangulation = BRep_Tool::Triangulation(face->GetFace(), location);
        bool valid = !triangulation.IsNull() && triangulation->HasUVNodes() &&
            triangulation->NbNodes() >= 3 && triangulation->NbNodes() <= 8192 &&
            triangulation->NbTriangles() > 0 && triangulation->NbTriangles() <= 16384 &&
            std::isfinite(triangulation->Deflection()) && triangulation->Deflection() >= 0.0 &&
            (face->GetStatusMask() & ~(IMeshData_Outdated | IMeshData_Failure)) == 0;
        spherical_retry_stop_ = "trial triangulation null " + std::to_string(triangulation.IsNull()) + " status " +
            std::to_string(face->GetStatusMask()) + " nodes/triangles " +
            std::to_string(triangulation.IsNull() ? 0 : triangulation->NbNodes()) + '/' +
            std::to_string(triangulation.IsNull() ? 0 : triangulation->NbTriangles());
        const auto finite = [](const gp_Pnt& point) {
          return std::isfinite(point.X()) && std::isfinite(point.Y()) && std::isfinite(point.Z());
        };
        const double deflection = GetParameters().Deflection;
        const double radius = face->GetSurface()->Sphere().Radius();
        valid &= std::isfinite(deflection) && deflection > 0.0 &&
            std::isfinite(radius) && radius > 0.0;
        std::map<std::pair<int, int>, int> links;
        std::set<std::pair<int, int>> boundary_links;
        double actual_uv_area = 0.0, uv_area_scale = 0.0;
        const auto link = [](int a, int b) { return std::make_pair(std::min(a, b), std::max(a, b)); };
        if (valid) {
          for (int i = 1; i <= triangulation->NbNodes() && valid; ++i) {
            spherical_retry_stop_ = "nonfinite node";
            const auto uv = triangulation->UVNode(i);
            valid &= finite(triangulation->Node(i)) && std::isfinite(uv.X()) && std::isfinite(uv.Y());
          }
          for (int ti = 1; ti <= triangulation->NbTriangles() && valid; ++ti) {
            spherical_retry_stop_ = "invalid triangle index";
            int ids[3]; triangulation->Triangle(ti).Get(ids[0], ids[1], ids[2]);
            for (int id : ids) valid &= id >= 1 && id <= triangulation->NbNodes();
            if (!valid || ids[0] == ids[1] || ids[1] == ids[2] || ids[2] == ids[0]) {
              valid = false; break;
            }
            gp_Pnt points[3]; gp_Pnt2d uv[3];
            double source_error = 0.0, diameter = 0.0;
            for (int i = 0; i < 3; ++i) {
              points[i] = triangulation->Node(ids[i]).Transformed(location.Transformation());
              uv[i] = triangulation->UVNode(ids[i]);
              const auto source = face->GetSurface()->Value(uv[i].X(), uv[i].Y());
              if (!finite(points[i]) || !finite(source)) { valid = false; break; }
              source_error = std::max(source_error, points[i].Distance(source));
              ++links[link(ids[i], ids[(i + 1) % 3])];
            }
            if (!valid) break;
            const double area_squared = gp_Vec(points[0], points[1]).Crossed(
                gp_Vec(points[0], points[2])).SquareMagnitude();
            spherical_retry_stop_ = "zero area or incorrect UV winding";
            valid &= std::isfinite(area_squared) && area_squared > 0.0;
            const double triangle_uv_area = 0.5 * (uv[1].Coord() - uv[0].Coord()).Crossed(
                uv[2].Coord() - uv[0].Coord());
            valid &= std::isfinite(triangle_uv_area) && triangle_uv_area != 0.0 &&
                std::signbit(triangle_uv_area) == std::signbit(expected_uv_area);
            actual_uv_area += std::abs(triangle_uv_area);
            for (int i = 0; i < 3; ++i)
              for (int j = i + 1; j < 3; ++j)
                diameter = std::max(diameter, std::abs(uv[i].X() - uv[j].X()) +
                    std::abs(uv[i].Y() - uv[j].Y()));
            uv_area_scale += diameter * diameter;
            // A sphere's directional second derivative is bounded by
            // radius*(|du|+|dv|)^2. This bounds interpolation error across the
            // entire triangle, including its original 3D boundary-node error.
            const double error_bound = source_error + 0.5 * radius * diameter * diameter;
            if (valid) spherical_retry_stop_ = "sphere deflection bound";
            valid &= std::isfinite(error_bound) && error_bound <= deflection;
          }
          for (const auto& boundary : boundaries) {
            for (int i = 0; i < boundary.pcurve->ParametersNb() && valid; ++i) {
              spherical_retry_stop_ = "shared boundary node mismatch";
              const int id = boundary.pcurve->GetIndex(i);
              if (id < 1 || id > triangulation->NbNodes()) { valid = false; break; }
              const auto point = triangulation->Node(id).Transformed(location.Transformation());
              const auto& original = boundary.curve->GetPoint(i);
              // No tolerance-sized replacement of a shared 3D boundary sample.
              valid &= finite(original) && point.Distance(original) <= Precision::Confusion() &&
                  triangulation->UVNode(id).Distance(boundary.pcurve->GetPoint(i)) <= Precision::PConfusion();
              if (i > 0) {
                const int previous = boundary.pcurve->GetIndex(i - 1);
                if (previous == id) {
                  valid &= original.Distance(boundary.curve->GetPoint(i - 1)) <= Precision::Confusion();
                } else boundary_links.insert(link(previous, id));
              }
            }
          }
          if (valid) spherical_retry_stop_ = "incomplete or nonmanifold boundary";
          for (const auto& entry : links)
            valid &= entry.second == (boundary_links.count(entry.first) ? 1 : 2);
          for (const auto& entry : boundary_links)
            valid &= links.count(entry) != 0 && links[entry] == 1;
          const double area_roundoff = 64.0 * std::numeric_limits<double>::epsilon() * uv_area_scale;
          if (valid) spherical_retry_stop_ = "UV domain coverage";
          valid &= std::isfinite(actual_uv_area) && std::isfinite(area_roundoff) &&
              std::abs(actual_uv_area - std::abs(expected_uv_area)) <= area_roundoff;
        }
        if (valid) spherical_retry_stop_ = "wire status or cancelled operation";
        for (const auto& wire : wires) valid &= wire->GetStatusMask() == 0;
        if (!valid || !range.More()) { rollback(); return false; }
        triangulation->Deflection(deflection);
        // The original generic Failure is cleared only after a complete,
        // conforming triangulation exists; other failure bits are never cleared.
        face->UnsetStatus(IMeshData_Failure);
        return true;
      } catch (const RollbackFailure&) {
        throw;
      } catch (const Standard_Failure&) {
        spherical_retry_stop_ = "OCCT exception";
        rollback(); return false;
      } catch (const std::exception&) {
        spherical_retry_stop_ = "exception";
        rollback(); return false;
      }
    } catch (const RollbackFailure&) {
      throw;
    } catch (const Standard_Failure&) {
      return false;
    } catch (const std::exception&) {
      return false;
    }
  }

  std::string boundary_repair_stop_ = "not run";
  std::string spherical_retry_stop_;
};

static std::string spherical_boundary_failure_detail(const IMeshData::IFaceHandle& face) {
  try {
    if (face->GetSurface()->GetType() != GeomAbs_Sphere) return "";
    std::ostringstream detail, samples, summary;
    detail.precision(10);
    samples.precision(9);
    summary.precision(10);
    struct Segment {
      gp_Pnt2d a, b;
      int edge, index;
      IMeshData::IEdgePtr native_edge;
      double first_parameter, last_parameter;
    };
    std::vector<Segment> segments;
    int edge_count = 0, sample_count = 0;
    bool complete = true;
    const auto finite_uv = [](const gp_Pnt2d& uv) {
      return std::isfinite(uv.X()) && std::isfinite(uv.Y());
    };
    for (int wi = 0; wi < face->WiresNb() && complete; ++wi) {
      const auto& wire = face->GetWire(wi);
      for (int ei = 0; ei < wire->EdgesNb() && complete; ++ei) {
        if (++edge_count > 6) { complete = false; break; }
        const auto& edge = wire->GetEdge(ei);
        const auto orientation = wire->GetEdgeOrientation(ei);
        const auto& pc = edge->GetPCurve(face.get(), orientation);
        const auto& curve = edge->GetCurve();
        if (pc.IsNull() || pc->ParametersNb() != curve->ParametersNb()) {
          complete = false; break;
        }
        BRepAdaptor_Curve on_face(TopoDS::Edge(edge->GetEdge().Oriented(orientation)), face->GetFace());
        const auto& source = on_face.CurveOnSurface().GetCurve();
        double maximum_mesh_error = 0.0, maximum_source_error = 0.0;
        samples << "; sphere wire/edge " << wi << '/' << ei << " boundary samples";
        for (int i = 0; i < pc->ParametersNb(); ++i) {
          if (++sample_count > 24) { complete = false; break; }
          const auto& uv = pc->GetPoint(i);
          const double parameter = pc->GetParameter(i);
          if (!finite_uv(uv) || !std::isfinite(parameter))
            return ", sphere sample diagnostic found nonfinite UV/parameter";
          samples << " [" << i << " t " << parameter << " UV " << uv.X() << ',' << uv.Y();
          const auto mesh_point = face->GetSurface()->Value(uv.X(), uv.Y());
          const double mesh_error = mesh_point.Distance(curve->GetPoint(i));
          if (!std::isfinite(mesh_error)) return ", sphere sample diagnostic found nonfinite distance";
          maximum_mesh_error = std::max(maximum_mesh_error, mesh_error);
          samples << " error mm " << mesh_error;
          if (edge->GetSameParam() && edge->GetSameRange() && !source.IsNull()) {
            const auto source_uv = source->Value(parameter);
            if (!finite_uv(source_uv)) return ", sphere sample diagnostic found nonfinite source UV";
            const double source_error = face->GetSurface()->Value(source_uv.X(), source_uv.Y()).Distance(curve->GetPoint(i));
            if (!std::isfinite(source_error)) return ", sphere sample diagnostic found nonfinite source distance";
            maximum_source_error = std::max(maximum_source_error, source_error);
            samples << " sourceUV " << source_uv.X() << ',' << source_uv.Y()
                   << " source error mm " << source_error;
          }
          samples << ']';
          if (i > 0) segments.push_back({pc->GetPoint(i - 1), uv, edge_count, i - 1,
              edge, pc->GetParameter(i - 1), parameter});
        }
        summary << ", sphere edge " << ei << " max mesh/source error mm "
                << maximum_mesh_error << '/' << maximum_source_error;
      }
    }
    int crossings = 0;
    for (std::size_t a = 0; a < segments.size(); ++a) {
      for (std::size_t b = a + 1; b < segments.size(); ++b) {
        const auto& first = segments[a]; const auto& second = segments[b];
        if (first.edge == second.edge && std::abs(first.index - second.index) <= 1) continue;
        gp_Pnt2d cross;
        if (BRepMesh_GeomTool::IntSegSeg(first.a.Coord(), first.b.Coord(),
                second.a.Coord(), second.b.Coord(), false, false, cross) != BRepMesh_GeomTool::Cross) continue;
        if (!finite_uv(cross)) return ", sphere sample diagnostic found nonfinite crossing";
        ++crossings;
        if (crossings <= 2) {
          detail << ", unfiltered cross edges/segments " << first.edge - 1 << '/' << first.index
                 << ':' << second.edge - 1 << '/' << second.index
                 << " UV " << cross.X() << ',' << cross.Y();
          if (!first.native_edge->GetSameParam() || !first.native_edge->GetSameRange() ||
              !second.native_edge->GetSameParam() || !second.native_edge->GetSameRange()) {
            detail << " native parameter correspondence unavailable";
            continue;
          }
          const auto av = first.b.Coord() - first.a.Coord();
          const auto bv = second.b.Coord() - second.a.Coord();
          const double aa = av.SquareModulus(), bb = bv.SquareModulus();
          if (!std::isfinite(aa) || !std::isfinite(bb) || aa <= 0.0 || bb <= 0.0) continue;
          const double at = first.first_parameter + (first.last_parameter - first.first_parameter) *
              (cross.Coord() - first.a.Coord()).Dot(av) / aa;
          const double bt = second.first_parameter + (second.last_parameter - second.first_parameter) *
              (cross.Coord() - second.a.Coord()).Dot(bv) / bb;
          if (!std::isfinite(at) || !std::isfinite(bt)) continue;
          BRepAdaptor_Curve ac(first.native_edge->GetEdge()), bc(second.native_edge->GetEdge());
          const auto ap = ac.Value(at), bp = bc.Value(bt);
          const auto cross_point = face->GetSurface()->Value(cross.X(), cross.Y());
          const double separation = ap.Distance(bp);
          if (!std::isfinite(separation) || !std::isfinite(cross_point.X()) ||
              !std::isfinite(cross_point.Y()) || !std::isfinite(cross_point.Z())) continue;
          detail << " native parameters " << at << '/' << bt
                 << " native separation mm " << separation;
          if (ac.GetType() == GeomAbs_Circle && bc.GetType() == GeomAbs_Circle &&
              std::isfinite(ac.Circle().Radius()) && std::isfinite(bc.Circle().Radius()) &&
              std::isfinite(ac.Circle().Location().Distance(bc.Circle().Location())))
            detail << " circle radii/center distance mm " << ac.Circle().Radius() << '/'
                   << bc.Circle().Radius() << '/' << ac.Circle().Location().Distance(bc.Circle().Location());
          TopoDS_Vertex a0, a1, b0, b1;
          TopExp::Vertices(first.native_edge->GetEdge(), a0, a1);
          TopExp::Vertices(second.native_edge->GetEdge(), b0, b1);
          for (const auto& vertex : {a0, a1}) {
            if (!vertex.IsNull() && ((!b0.IsNull() && vertex.IsSame(b0)) ||
                                    (!b1.IsNull() && vertex.IsSame(b1))) &&
                std::isfinite(cross_point.Distance(BRep_Tool::Pnt(vertex))) &&
                std::isfinite(BRep_Tool::Tolerance(vertex)))
              detail << " shared vertex distance/tolerance mm "
                     << cross_point.Distance(BRep_Tool::Pnt(vertex)) << '/'
                     << BRep_Tool::Tolerance(vertex);
          }
        }
      }
    }
    if (!complete) detail << " (24-point diagnostic scope limited)";
    return ", unfiltered sphere crossings " + std::to_string(crossings) +
        detail.str().substr(0, 1500) + summary.str().substr(0, 600) + samples.str().substr(0, 1500);
  } catch (const Standard_Failure&) {
    return ", sphere sample diagnostic unavailable (OCCT exception)";
  } catch (const std::exception&) {
    return ", sphere sample diagnostic unavailable (exception)";
  }
}

// Optional diagnostics of a failed face's actual meshing domain. This does not
// change its shared samples, exact geometry or failure status.
static std::string face_mesh_failure_detail(
    const IMeshData::IFaceHandle& face, const IMeshTools_Parameters& parameters) {
  try {
    std::ostringstream detail;
    detail.precision(12);
    TCollection_AsciiString algorithm = OSD_Environment("CSF_MeshAlgo").Value();
    algorithm.LowerCase();
    detail << ", default algorithm "
           << ((algorithm == "delabella" || algorithm == "1") ? "Delabella" : "Watson")
           << ", face orientation " << face->GetFace().Orientation()
           << " tolerance/deflection mm " << BRep_Tool::Tolerance(face->GetFace())
           << '/' << face->GetDeflection()
           << ", requested deflection/min size mm " << parameters.Deflection
           << '/' << parameters.MinSize << " angle " << parameters.Angle
           << " adjust min size " << parameters.AdjustMinSize;
    const auto& surface = face->GetSurface();
    if (surface->GetType() == GeomAbs_Sphere)
      detail << ", sphere radius mm " << surface->Sphere().Radius();
    double u0, u1, v0, v1;
    BRepTools::UVBounds(face->GetFace(), u0, u1, v0, v1);
    if (!std::isfinite(u0) || !std::isfinite(u1) ||
        !std::isfinite(v0) || !std::isfinite(v1))
      return ", face mesh diagnostic found nonfinite UV bounds";
    detail << ", source UV bounds " << u0 << ':' << u1 << '/' << v0 << ':' << v1;
    BRepMesh_SphereRangeSplitter splitter;
    splitter.Reset(face, parameters);
    std::vector<gp_Pnt> points;
    gp_Pnt2d uv_min(std::numeric_limits<double>::max(), std::numeric_limits<double>::max());
    gp_Pnt2d uv_max(-std::numeric_limits<double>::max(), -std::numeric_limits<double>::max());
    gp_Pnt xyz_min(std::numeric_limits<double>::max(), std::numeric_limits<double>::max(),
                   std::numeric_limits<double>::max());
    gp_Pnt xyz_max(-std::numeric_limits<double>::max(), -std::numeric_limits<double>::max(),
                   -std::numeric_limits<double>::max());
    int edge_count = 0;
    bool complete = true;
    for (int wi = 0; wi < face->WiresNb(); ++wi) {
      const auto& wire = face->GetWire(wi);
      std::vector<gp_Pnt2d> polygon;
      for (int ei = 0; ei < wire->EdgesNb(); ++ei) {
        if (++edge_count > 6) { complete = false; break; }
        const auto& edge = wire->GetEdge(ei);
        const auto orientation = wire->GetEdgeOrientation(ei);
        const auto& pc = edge->GetPCurve(face.get(), orientation);
        if (pc.IsNull()) { complete = false; continue; }
        const auto& curve = edge->GetCurve();
        if (curve->ParametersNb() != pc->ParametersNb()) { complete = false; continue; }
        const int count = pc->ParametersNb();
        detail << "; edge " << ei << " kind " << BRepAdaptor_Curve(edge->GetEdge()).GetType()
               << " orientation " << orientation << " tolerance mm "
               << BRep_Tool::Tolerance(edge->GetEdge()) << " degenerate "
               << BRep_Tool::Degenerated(edge->GetEdge());
        for (int i = 0; i < count; ++i) {
          if (points.size() >= 2048) { complete = false; break; }
          const auto& uv = pc->GetPoint(i);
          const auto& point = curve->GetPoint(i);
          if (!std::isfinite(uv.X()) || !std::isfinite(uv.Y()) ||
              !std::isfinite(point.X()) || !std::isfinite(point.Y()) ||
              !std::isfinite(point.Z()))
            return ", face mesh diagnostic found nonfinite boundary samples";
          splitter.AddPoint(uv);
          uv_min.SetX(std::min(uv_min.X(), uv.X()));
          uv_min.SetY(std::min(uv_min.Y(), uv.Y()));
          uv_max.SetX(std::max(uv_max.X(), uv.X()));
          uv_max.SetY(std::max(uv_max.Y(), uv.Y()));
          for (int axis = 1; axis <= 3; ++axis) {
            xyz_min.SetCoord(axis, std::min(xyz_min.Coord(axis), point.Coord(axis)));
            xyz_max.SetCoord(axis, std::max(xyz_max.Coord(axis), point.Coord(axis)));
          }
          points.push_back(point);
          if (i == 0 || i == count / 2 || i == count - 1)
            detail << " UV[" << i << "] " << uv.X() << ',' << uv.Y();
        }
        // The oriented wire polygon excludes each edge's duplicate ending node,
        // matching NodeInsertionMeshAlgo's collection of boundary points.
        for (int i = 0; i < count - 1 && polygon.size() < 2048; ++i)
          polygon.push_back(pc->GetPoint(orientation == TopAbs_REVERSED ? count - 1 - i : i));
        if (!complete) break;
      }
      if (polygon.size() >= 3) {
        double twice_area = 0.0;
        const auto origin = polygon.front().Coord();
        for (std::size_t i = 0; i < polygon.size(); ++i)
          twice_area += (polygon[i].Coord() - origin).Crossed(
              polygon[(i + 1) % polygon.size()].Coord() - origin);
        if (std::isfinite(twice_area))
          detail << ", wire " << wi << " signed UV area " << 0.5 * twice_area;
      }
      if (!complete) break;
    }
    if (!points.empty()) {
      double minimum_gap = std::numeric_limits<double>::max();
      int coincident_pairs = 0;
      for (std::size_t i = 0; i < points.size(); ++i) {
        for (std::size_t j = i + 1; j < points.size(); ++j) {
          const double gap = points[i].Distance(points[j]);
          if (!std::isfinite(gap)) return ", face mesh diagnostic found nonfinite distance";
          if (gap == 0.0) ++coincident_pairs;
          else minimum_gap = std::min(minimum_gap, gap);
        }
      }
      detail << ", sampled UV bounds " << uv_min.X() << ':' << uv_max.X()
             << '/' << uv_min.Y() << ':' << uv_max.Y()
             << ", boundary XYZ span mm " << xyz_max.X() - xyz_min.X() << ','
             << xyz_max.Y() - xyz_min.Y() << ',' << xyz_max.Z() - xyz_min.Z();
      if (minimum_gap != std::numeric_limits<double>::max())
        detail << " min positive gap mm " << minimum_gap;
      detail << " coincident pairs " << coincident_pairs;
      if (complete && surface->GetType() == GeomAbs_Sphere) {
        splitter.AdjustRange();
        detail << ", sphere splitter valid " << splitter.IsValid()
               << " UV tolerance " << splitter.GetToleranceUV().first << '/'
               << splitter.GetToleranceUV().second << " delta "
               << splitter.GetDelta().first << '/' << splitter.GetDelta().second;
      }
    }
    if (!complete) detail << ", face mesh diagnostic scope limited";
    return detail.str().substr(0, 2200);
  } catch (const Standard_Failure&) {
    return ", face mesh diagnostic unavailable (OCCT exception)";
  } catch (const std::exception&) {
    return ", face mesh diagnostic unavailable (exception)";
  }
}

static std::string boundary_failure_detail(
    const IMeshData::IFaceHandle& face, const IMeshTools_Parameters& parameters) {
  try {
    BRepMesh_FaceChecker checker(face, parameters);
    if (checker.Perform()) return ", no reported boundary intersections";
    const auto& intersections = checker.GetIntersectingEdges();
    if (intersections.IsNull()) return ", no crossing edge map";
    struct Edge {
      IMeshData::IEdgePtr edge;
      IMeshData::IPCurveHandle pcurve;
      int wire;
      int index;
    };
    std::vector<Edge> edges;
    for (int wi = 0; wi < face->WiresNb() && edges.size() < 6; ++wi) {
      const auto& wire = face->GetWire(wi);
      for (int ei = 0; ei < wire->EdgesNb() && edges.size() < 6; ++ei) {
        auto edge = wire->GetEdge(ei);
        if (!intersections->Contains(edge)) continue;
        const auto& pcurve = edge->GetPCurve(face.get(), wire->GetEdgeOrientation(ei));
        if (!pcurve.IsNull() && pcurve->ParametersNb() >= 2)
          edges.push_back({edge, pcurve, wi, ei});
      }
    }
    std::size_t comparisons = 0;
    const auto finite_uv = [](const gp_Pnt2d& point) {
      return std::isfinite(point.X()) && std::isfinite(point.Y());
    };
    for (std::size_t a = 0; a < edges.size(); ++a) {
      for (std::size_t b = a + 1; b < edges.size(); ++b) {
        const auto& ae = edges[a]; const auto& be = edges[b];
        for (int ai = 1; ai < ae.pcurve->ParametersNb(); ++ai) {
          const auto& p = ae.pcurve->GetPoint(ai - 1);
          const auto& q = ae.pcurve->GetPoint(ai);
          for (int bi = 1; bi < be.pcurve->ParametersNb(); ++bi) {
            if (++comparisons > 16000000) return ", crossing diagnostic search limit";
            const auto& r = be.pcurve->GetPoint(bi - 1);
            const auto& s = be.pcurve->GetPoint(bi);
            if (!finite_uv(p) || !finite_uv(q) || !finite_uv(r) || !finite_uv(s))
              return ", boundary diagnostic found nonfinite UV samples";
            if (std::max(p.X(), q.X()) < std::min(r.X(), s.X()) ||
                std::max(r.X(), s.X()) < std::min(p.X(), q.X()) ||
                std::max(p.Y(), q.Y()) < std::min(r.Y(), s.Y()) ||
                std::max(r.Y(), s.Y()) < std::min(p.Y(), q.Y())) continue;
            gp_Pnt2d intersection;
            if (BRepMesh_GeomTool::IntSegSeg(p.Coord(), q.Coord(), r.Coord(), s.Coord(),
                    false, false, intersection) != BRepMesh_GeomTool::Cross) continue;
            if (!finite_uv(intersection))
              return ", boundary diagnostic found nonfinite intersection";
            const gp_XY av = q.Coord() - p.Coord(), bv = s.Coord() - r.Coord();
            const double aa = av.SquareModulus(), bb = bv.SquareModulus();
            const double dot = av.Dot(bv);
            if (!std::isfinite(aa) || !std::isfinite(bb) || !std::isfinite(dot) ||
                aa <= 0.0 || bb <= 0.0) continue;
            const double cosine = (dot / std::sqrt(aa)) / std::sqrt(bb);
            if (!std::isfinite(cosine)) continue;
            const double angle = std::acos(std::clamp(cosine, -1.0, 1.0));
            if (angle < kPi / 36.0) continue;
            const double af = (intersection.Coord() - p.Coord()).Dot(av) / av.SquareModulus();
            const double bf = (intersection.Coord() - r.Coord()).Dot(bv) / bv.SquareModulus();
            const double at = ae.pcurve->GetParameter(ai - 1) +
                af * (ae.pcurve->GetParameter(ai) - ae.pcurve->GetParameter(ai - 1));
            const double bt = be.pcurve->GetParameter(bi - 1) +
                bf * (be.pcurve->GetParameter(bi) - be.pcurve->GetParameter(bi - 1));
            if (!std::isfinite(at) || !std::isfinite(bt))
              return ", boundary diagnostic found nonfinite curve parameters";
            std::ostringstream detail;
            detail.precision(12);
            detail << ", crossing wires/edges " << ae.wire << '/' << ae.index
                   << ':' << be.wire << '/' << be.index << " segments "
                   << ai - 1 << ':' << bi - 1 << " UV " << intersection.X()
                   << ',' << intersection.Y() << " angle deg " << angle * 180.0 / kPi
                   << " parameters " << at << ':' << bt
                   << " intervals " << ae.pcurve->GetParameter(ai - 1) << ':'
                   << ae.pcurve->GetParameter(ai) << '/'
                   << be.pcurve->GetParameter(bi - 1) << ':' << be.pcurve->GetParameter(bi)
                   << " segment UV " << p.X() << ',' << p.Y() << ':' << q.X() << ',' << q.Y()
                   << '/' << r.X() << ',' << r.Y() << ':' << s.X() << ',' << s.Y()
                   << " face deflection " << face->GetDeflection();
            const gp_Pnt cross_point = face->GetSurface()->Value(intersection.X(), intersection.Y());
            TopoDS_Vertex vertex;
            const bool has_vertex = TopExp::CommonVertex(ae.edge->GetEdge(), be.edge->GetEdge(), vertex);
            if (has_vertex) {
              detail << " vertex distance/tolerance mm "
                     << cross_point.Distance(BRep_Tool::Pnt(vertex)) << '/'
                     << BRep_Tool::Tolerance(vertex);
            }
            for (const auto& item : {std::make_pair(ae, at), std::make_pair(be, bt)}) {
              const auto& edge = item.first;
              const auto& pc = edge.pcurve;
              detail << "; edge " << edge.index << " orientation " << pc->GetOrientation()
                     << " same param/range " << edge.edge->GetSameParam() << '/'
                     << edge.edge->GetSameRange() << " tolerance mm "
                     << BRep_Tool::Tolerance(edge.edge->GetEdge());
              if (!edge.edge->GetSameParam() || !edge.edge->GetSameRange()) continue;
              BRepAdaptor_Curve native(edge.edge->GetEdge());
              BRepAdaptor_Curve on_face(
                  TopoDS::Edge(edge.edge->GetEdge().Oriented(pc->GetOrientation())), face->GetFace());
              const auto& exact_pc = on_face.CurveOnSurface().GetCurve();
              detail << " endpoints";
              for (int index : {0, pc->ParametersNb() - 1}) {
                const auto& uv = pc->GetPoint(index);
                const double t = pc->GetParameter(index);
                if (!finite_uv(uv) || !std::isfinite(t))
                  return ", boundary diagnostic found nonfinite endpoint";
                const auto native_uv = exact_pc->Value(t);
                if (!finite_uv(native_uv))
                  return ", boundary diagnostic found nonfinite source endpoint";
                const auto native_point = native.Value(t);
                detail << " [" << t << " UV " << uv.X() << ',' << uv.Y()
                       << " sourceUV " << native_uv.X() << ',' << native_uv.Y()
                       << " shift/error mm "
                       << face->GetSurface()->Value(uv.X(), uv.Y()).Distance(
                              face->GetSurface()->Value(native_uv.X(), native_uv.Y())) << '/'
                       << face->GetSurface()->Value(uv.X(), uv.Y()).Distance(native_point)
                       << " source error mm "
                       << face->GetSurface()->Value(native_uv.X(), native_uv.Y()).Distance(native_point) << ']';
              }
              const auto native_uv = exact_pc->Value(item.second);
              if (!finite_uv(native_uv))
                return ", boundary diagnostic found nonfinite source intersection";
              const auto native_point = native.Value(item.second);
              detail << " crossing chord/source error mm " << cross_point.Distance(native_point)
                     << '/' << face->GetSurface()->Value(native_uv.X(), native_uv.Y()).Distance(native_point);
              if (has_vertex)
                detail << " native vertex distance mm " << native_point.Distance(BRep_Tool::Pnt(vertex));
            }
            return detail.str().substr(0, 2400);
          }
        }
      }
    }
    return ", no different-edge crossing found in diagnostic scope";
  } catch (const Standard_Failure&) {
    return ", boundary diagnostic unavailable (OCCT failure)";
  } catch (const std::exception&) {
    return ", boundary diagnostic unavailable (native exception)";
  }
}

static FfiMesh mesh_shape(std::uint64_t body_id,
                          const TopoDS_Shape& shape,
                          double linear_deflection,
                          double angular_deflection,
                          bool imported_display = false,
                          const SectionMeshBudget* budget = nullptr,
                          const Message_ProgressRange& range = Message_ProgressRange());

// Shared exact clipping for drawing projections and disposable 3D inspection.
static TopoDS_Shape retain_half_space(const TopoDS_Shape& source,
                                     const gp_Pln& boundary,
                                     const gp_Pnt& retained_point,
                                     const Message_ProgressRange& range = Message_ProgressRange()) {
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(boundary).Face();
  const TopoDS_Solid half_space =
      BRepPrimAPI_MakeHalfSpace(face, retained_point).Solid();
  BRepAlgoAPI_Common common;
  TopTools_ListOfShape arguments, tools;
  arguments.Append(source);
  tools.Append(half_space);
  common.SetArguments(arguments);
  common.SetTools(tools);
  common.SetNonDestructive(true);
  common.Build(range);
  if (!common.IsDone() || common.HasErrors()) {
    std::ostringstream errors;
    common.DumpErrors(errors);
    throw std::runtime_error("OCCT section clipping failed: " + errors.str());
  }
  return common.Shape();
}

static TopoDS_Shape exact_section_shape(const TopoDS_Shape& source,
                                      const gp_Pln& plane,
                                      const Message_ProgressRange& range = Message_ProgressRange()) {
  BRepAlgoAPI_Section section(source, plane, false);
  section.SetNonDestructive(true);
  section.Approximation(true);
  section.Build(range);
  if (!section.IsDone() || section.HasErrors()) {
    std::ostringstream errors;
    section.DumpErrors(errors);
    throw std::runtime_error("OCCT section intersection failed: " + errors.str());
  }
  return section.Shape();
}

FfiSectionGeometry Kernel::section_geometry(std::uint64_t body_id,
                                            const FfiSectionOptions& options) const {
  const auto found = impl_->bodies.find(body_id);
  const auto axis = options.axis;
  const auto offset = options.offset;
  if (found == impl_->bodies.end() || axis > 2 || !std::isfinite(offset) ||
      !std::isfinite(options.deflection) || options.deflection < 0.001 ||
      options.deflection > 0.1 || options.timeout_ms > 30'000) {
    throw std::runtime_error("Invalid body or section plane");
  }
  Handle(SectionProgress) progress = new SectionProgress(options.timeout_ms);
  progress->check("dispatch");
  Message_ProgressScope stages(progress->Start(), "Section inspection", 3);
  double coordinates[3] = {0., 0., 0.};
  coordinates[axis] = offset;
  const gp_Pnt point(coordinates[0], coordinates[1], coordinates[2]);
  double normal[3] = {0., 0., 0.};
  normal[axis] = 1.;
  const gp_Vec direction(normal[0], normal[1], normal[2]);
  const gp_Pln plane(point, gp_Dir(direction));
  const TopoDS_Shape section = exact_section_shape(found->second, plane, stages.Next());
  progress->check("intersection");
  const gp_Vec right = axis == 0 ? gp_Vec(0., 1., 0.) : gp_Vec(1., 0., 0.);
  const gp_Vec up = axis == 2 ? gp_Vec(0., 1., 0.) : gp_Vec(0., 0., 1.);
  FfiSectionGeometry output;
  output.outcome = 0;
  output.has_cutaway = false;
  output.cutaway.body_id = body_id;
  output.offsets.push_back(0);
  std::set<std::vector<std::int64_t>> seen;
  append_section_shape(section, right, up, options.deflection, output.offsets,
                       output.points, seen, progress.get(), options.contour_points);
  if (output.points.empty()) {
    output.outcome = TopExp_Explorer(section, TopAbs_VERTEX).More() ? 1 : 0;
    return output;
  }

  TopoDS_Shape source = found->second;
  if (options.include_cutaway) {
    BRepBuilderAPI_Copy copy(source, true, false);
    if (!copy.IsDone() || copy.Shape().IsNull()) {
      throw std::runtime_error("OCCT section shape copy failed");
    }
    source = copy.Shape();
  }
  TopTools_IndexedMapOfShape solids;
  for (TopExp_Explorer explorer(source, TopAbs_SOLID); explorer.More(); explorer.Next()) {
    progress->check("solid classification");
    if (static_cast<std::size_t>(solids.Extent()) >= options.contour_points) {
      throw std::runtime_error("Section source exceeds the native solid budget");
    }
    solids.Add(explorer.Current());
  }
  if (solids.IsEmpty()) throw std::runtime_error("Section inspection requires solid geometry");
  Message_ProgressScope clipping(stages.Next(), "Clip section solids", solids.Extent());
  BRep_Builder builder;
  TopoDS_Compound clipped;
  builder.MakeCompound(clipped);
  bool splits_material = false;
  for (int index = 1; index <= solids.Extent(); ++index) {
    const auto& solid = solids.FindKey(index);
    const auto retained = retain_half_space(solid, plane,
        point.Translated(direction.Multiplied(options.keep_positive ? 1. : -1.)), clipping.Next());
    progress->check("clipping");
    GProp_GProps source_properties, retained_properties;
    BRepGProp::VolumeProperties(solid, source_properties);
    if (!retained.IsNull()) BRepGProp::VolumeProperties(retained, retained_properties);
    const double source_volume = std::abs(source_properties.Mass());
    const double retained_volume = std::abs(retained_properties.Mass());
    if (!std::isfinite(source_volume) || source_volume <= 0.0 || !std::isfinite(retained_volume)) {
      throw std::runtime_error("Section inspection requires a finite solid volume");
    }
    const double tolerance = std::max(1e-15, source_volume * 1e-12);
    splits_material |= retained_volume > tolerance && source_volume - retained_volume > tolerance;
    if (retained_volume > tolerance) builder.Add(clipped, retained);
  }
  output.outcome = splits_material ? 2 : 1;
  if (output.outcome == 2 && options.include_cutaway) {
    const SectionMeshBudget budget{options.vertices, options.edge_points, progress.get()};
    output.cutaway = mesh_shape(body_id, clipped, options.deflection, 0.25, false, &budget, stages.Next());
    progress->check("meshing");
    if (output.cutaway.indices.empty()) {
      throw std::runtime_error("OCCT produced no triangles for the retained section solid");
    }
    output.has_cutaway = true;
  }
  return output;
}

FfiMesh Kernel::mesh(std::uint64_t body_id) const {
  const auto found = impl_->bodies.find(body_id);
  if (found == impl_->bodies.end()) {
    throw std::runtime_error("body is missing");
  }

  return mesh_shape(body_id, found->second, 0.15, 0.35,
      impl_->imported_display_bodies.count(body_id) != 0);
}

FfiMesh Kernel::mesh_with_deflection(
    std::uint64_t body_id,
    double linear_deflection,
    double angular_deflection) const {
  const auto found = impl_->bodies.find(body_id);
  if (found == impl_->bodies.end()) {
    throw std::runtime_error("body is missing");
  }




  BRepBuilderAPI_Copy copy(found->second, true, false);
  if (!copy.IsDone() || copy.Shape().IsNull()) {
    throw std::runtime_error("OCCT export shape copy failed");
  }
  return mesh_shape(body_id, copy.Shape(), linear_deflection,
                    angular_deflection);
}

static FfiMesh mesh_shape(std::uint64_t body_id,
                          const TopoDS_Shape& shape,
                          double linear_deflection,
                          double angular_deflection,
                          bool imported_display,
                          const SectionMeshBudget* budget,
                          const Message_ProgressRange& range) {
  const double linear =
      linear_deflection > 0.0 ? linear_deflection : 0.15;
  const double angular =
      angular_deflection > 0.0 ? angular_deflection : 0.35;
  BRepMesh_IncrementalMesh mesher;
  mesher.SetShape(shape);
  mesher.ChangeParameters().Deflection = linear;
  mesher.ChangeParameters().Angle = angular;
  mesher.ChangeParameters().InParallel = true;
  auto* boundary_context = new TangentBoundaryMeshContext();
  Handle(IMeshTools_Context) context = boundary_context;
  mesher.Perform(context, range);
  if (budget) budget->progress->check("meshing");

  FfiMesh output;
  output.body_id = body_id;
  output.topology_signature = topology_signature(shape);
  TopTools_IndexedMapOfShape face_map;
  TopExp::MapShapes(shape, TopAbs_FACE, face_map);
  TopTools_IndexedMapOfShape edge_map;
  TopExp::MapShapes(shape, TopAbs_EDGE, edge_map);
  if (budget && (static_cast<std::size_t>(face_map.Extent()) > budget->vertices ||
                 static_cast<std::size_t>(edge_map.Extent()) > budget->edge_points)) {
    throw std::runtime_error("Section topology exceeds the native geometry budget");
  }
  for (int face_index = 1; face_index <= face_map.Extent(); ++face_index) {
    if (budget) budget->progress->check("mesh validation");
    const TopoDS_Face face = TopoDS::Face(face_map.FindKey(face_index));
    TopLoc_Location location;
    const Handle(Poly_Triangulation) triangulation =
        BRep_Tool::Triangulation(face, location);
    if (triangulation.IsNull() || triangulation->NbTriangles() == 0 || triangulation->NbNodes() < 3) {
      GProp_GProps properties;
      BRepGProp::SurfaceProperties(face, properties);
      if (!std::isfinite(properties.Mass()) || std::abs(properties.Mass()) > 1e-14) {
        std::ostringstream diagnostic;
        diagnostic.precision(17);
        diagnostic << "area " << properties.Mass()
                   << ", mesh status " << mesher.GetStatusFlags();
        const auto& model = context->GetModel();
        if (!model.IsNull()) {
          for (int fi = 0; fi < model->FacesNb(); ++fi) {
            const auto& discrete_face = model->GetFace(fi);
            if (!discrete_face->GetFace().IsSame(face)) continue;
            const auto surface_name = [](GeomAbs_SurfaceType type) {
              switch (type) {
                case GeomAbs_Plane: return "plane";
                case GeomAbs_Cylinder: return "cylinder";
                case GeomAbs_Cone: return "cone";
                case GeomAbs_Sphere: return "sphere";
                case GeomAbs_Torus: return "torus";
                case GeomAbs_BezierSurface: return "Bezier";
                case GeomAbs_BSplineSurface: return "B-spline";
                case GeomAbs_SurfaceOfRevolution: return "revolution";
                case GeomAbs_SurfaceOfExtrusion: return "extrusion";
                case GeomAbs_OffsetSurface: return "offset";
                default: return "other";
              }
            };
            diagnostic << ", surface "
                       << surface_name(discrete_face->GetSurface()->GetType())
                       << ", face status " << discrete_face->GetStatusMask()
                       << ", wires " << discrete_face->WiresNb()
                       << ", boundary repair " << boundary_context->BoundaryRepairStop();
            int reported_edges = 0;
            for (int wi = 0; wi < std::min(2, discrete_face->WiresNb()); ++wi) {
              const auto& wire = discrete_face->GetWire(wi);
              diagnostic << ", wire " << wi << " status " << wire->GetStatusMask()
                         << " edges " << wire->EdgesNb() << " samples";
              for (int ei = 0; ei < wire->EdgesNb() && reported_edges < 6;
                   ++ei, ++reported_edges) {
                const auto& pcurve = wire->GetEdge(ei)->GetPCurve(
                    discrete_face.get(), wire->GetEdgeOrientation(ei));
                diagnostic << ' ' << (pcurve.IsNull() ? 0 : pcurve->ParametersNb());
              }
            }
            if (!imported_display) diagnostic << face_mesh_failure_detail(discrete_face, context->GetParameters())
                       << spherical_boundary_failure_detail(discrete_face)
                       << boundary_failure_detail(discrete_face, context->GetParameters());
            break;
          }
        }
        const std::string failure = "OCCT did not triangulate body " +
            std::to_string(body_id) + " face " + std::to_string(face_index - 1) +
            " (" + diagnostic.str() + ")";
        if (!imported_display) throw std::runtime_error(failure);
        output.display_warning_face_indices.push_back(static_cast<std::uint32_t>(face_index - 1));
        output.display_warning_messages.push_back(rust::String(
            "Imported STEP face has no display triangles; exact geometry is retained. " +
            failure.substr(0, 1024)));
      }
    }
  }
  context->ChangeParameters().CleanModel = true;
  context->Clean();
  context.Nullify();
  output.face_edge_offsets.push_back(0);
  for (int face_index = 1; face_index <= face_map.Extent(); ++face_index) {
    if (budget) budget->progress->check("mesh extraction");
    const TopoDS_Face face = TopoDS::Face(face_map.FindKey(face_index));
    // Exact face slots and boundary keys survive absent display triangles.
    append_plane(output.face_plane_data, face);
    append_face_signature(output.face_signature_data, face);
    append_cylinder(output.face_cylinder_data, face);
    BRepAdaptor_Surface surface(face, true);
    if (surface.GetType() == GeomAbs_Cone) {
      const gp_Cone cone = surface.Cone();
      output.face_cone_data.push_back(1.0);
      output.face_cone_data.push_back(cone.Axis().Direction().X());
      output.face_cone_data.push_back(cone.Axis().Direction().Y());
      output.face_cone_data.push_back(cone.Axis().Direction().Z());
      output.face_cone_data.push_back(cone.SemiAngle());
    } else {
      for (int i = 0; i < 5; ++i) output.face_cone_data.push_back(0.0);
    }
    TopTools_IndexedMapOfShape boundary;
    TopExp::MapShapes(face, TopAbs_EDGE, boundary);
    for (int i = 1; i <= boundary.Extent(); ++i) {
      const int index = edge_map.FindIndex(boundary.FindKey(i));
      if (index <= 0) throw std::runtime_error("face boundary edge is absent from body topology");
      output.face_edge_indices.push_back(static_cast<std::uint32_t>(index - 1));
    }
    output.face_edge_offsets.push_back(static_cast<std::uint32_t>(output.face_edge_indices.size()));
    TopLoc_Location location;
    const Handle(Poly_Triangulation) triangulation =
        BRep_Tool::Triangulation(face, location);
    if (triangulation.IsNull() || triangulation->NbTriangles() == 0 || triangulation->NbNodes() < 3) {
      output.face_first_indices.push_back(static_cast<std::uint32_t>(output.indices.size()));
      output.face_index_counts.push_back(0);
      continue;
    }
    if (budget && (static_cast<std::size_t>(triangulation->NbNodes()) > budget->vertices ||
        static_cast<std::size_t>(triangulation->NbTriangles()) * 3 >
          budget->vertices - output.positions.size() / 3)) {
      throw std::runtime_error("Section mesh exceeds the native vertex budget");
    }
    if (!triangulation->HasNormals()) {
      triangulation->ComputeNormals();
    }
    output.face_first_indices.push_back(
        static_cast<std::uint32_t>(output.indices.size()));
    const gp_Trsf transform = location.Transformation();
    for (int triangle_index = 1;
         triangle_index <= triangulation->NbTriangles(); ++triangle_index) {
      const Poly_Triangle triangle = triangulation->Triangle(triangle_index);
      int indices[3] = {triangle.Value(1), triangle.Value(2),
                        triangle.Value(3)};
      if (face.Orientation() == TopAbs_REVERSED) {
        std::swap(indices[1], indices[2]);
      }
      gp_Pnt points[3] = {triangulation->Node(indices[0]).Transformed(transform),
                          triangulation->Node(indices[1]).Transformed(transform),
                          triangulation->Node(indices[2]).Transformed(transform)};
      gp_Vec triangle_normal(points[0], points[1]);
      triangle_normal.Cross(gp_Vec(points[0], points[2]));
      if (triangle_normal.SquareMagnitude() <= 1e-24) {
        continue;
      }
      for (int vertex = 0; vertex < 3; ++vertex) {
        gp_Dir normal = triangulation->Normal(indices[vertex]);
        normal.Transform(transform);
        if (face.Orientation() == TopAbs_REVERSED) {
          normal.Reverse();
        }
        output.positions.push_back(static_cast<float>(points[vertex].X()));
        output.positions.push_back(static_cast<float>(points[vertex].Y()));
        output.positions.push_back(static_cast<float>(points[vertex].Z()));
        append_vec(output.normals, gp_Vec(normal));
        output.indices.push_back(
            static_cast<std::uint32_t>(output.indices.size()));
      }
    }
    output.face_index_counts.push_back(
        static_cast<std::uint32_t>(output.indices.size()) -
        output.face_first_indices.back());
    if (output.face_index_counts.back() == 0) {
      GProp_GProps properties;
      BRepGProp::SurfaceProperties(face, properties);
      if (!std::isfinite(properties.Mass()) || std::abs(properties.Mass()) > 1e-14) {
        const std::string failure = "OCCT triangulation has no usable triangles for body " +
            std::to_string(body_id) + " face " + std::to_string(face_index - 1);
        if (!imported_display) throw std::runtime_error(failure);
        output.display_warning_face_indices.push_back(static_cast<std::uint32_t>(face_index - 1));
        output.display_warning_messages.push_back(rust::String(
            "Imported STEP face has no usable display triangles; exact geometry is retained. " + failure));
      }
    }
  }

  output.edge_point_offsets.push_back(0);
  if (imported_display && output.indices.empty())
    throw std::runtime_error("Imported STEP has no valid display triangles; exact geometry cannot be displayed");
  TopTools_IndexedDataMapOfShapeListOfShape edge_faces;
  TopExp::MapShapesAndUniqueAncestors(shape, TopAbs_EDGE, TopAbs_FACE,
                                      edge_faces, false);
  for (int edge_index = 1; edge_index <= edge_map.Extent(); ++edge_index) {
    if (budget) budget->progress->check("edge extraction");
    const TopoDS_Edge edge = TopoDS::Edge(edge_map.FindKey(edge_index));
    bool refinable = false;
    if (edge_faces.Contains(edge)) {
      const TopTools_ListOfShape& adjacent_faces =
          edge_faces.FindFromKey(edge);
      if (adjacent_faces.Extent() == 2) {
        TopTools_ListIteratorOfListOfShape iterator(adjacent_faces);
        const TopoDS_Face first_face = TopoDS::Face(iterator.Value());
        iterator.Next();
        const TopoDS_Face second_face = TopoDS::Face(iterator.Value());
        refinable =
            BRep_Tool::Continuity(edge, first_face, second_face) == GeomAbs_C0;
      }
    }
    output.edge_refinable.push_back(refinable ? 1 : 0);
    BRepAdaptor_Curve curve(edge);
    append_circle(output.edge_circle_data, edge, curve);
    if (budget) {
      const auto points = sample_section_edge(edge, 0.01,
          budget->edge_points - output.edge_points.size() / 3, *budget->progress);
      for (const auto& point : points) append_point(output.edge_points, point);
    } else if (curve.GetType() == GeomAbs_Line) {
      append_point(output.edge_points, curve.Value(curve.FirstParameter()));
      append_point(output.edge_points, curve.Value(curve.LastParameter()));
    } else {


      GCPnts_UniformDeflection discretization(curve, 0.01, true);
      if (discretization.IsDone() && discretization.NbPoints() >= 2) {
        for (int point_index = 1;
             point_index <= discretization.NbPoints(); ++point_index) {
          append_point(output.edge_points, discretization.Value(point_index));
        }
      } else {
        const double first = curve.FirstParameter();
        const double last = curve.LastParameter();
        constexpr int sample_count = 25;
        for (int sample = 0; sample < sample_count; ++sample) {
          const double t =
              first + (last - first) * static_cast<double>(sample) /
                          static_cast<double>(sample_count - 1);
          append_point(output.edge_points, curve.Value(t));
        }
      }
    }
    output.edge_point_offsets.push_back(
        static_cast<std::uint32_t>(output.edge_points.size() / 3));
  }
  return output;
}

FfiInterferenceResult Kernel::exact_interference(
    const FfiBodyPlacement& placement_a,
    const FfiBodyPlacement& placement_b) const {
  const auto found_a = impl_->bodies.find(placement_a.body_id);
  const auto found_b = impl_->bodies.find(placement_b.body_id);
  if (found_a == impl_->bodies.end() || found_b == impl_->bodies.end()) {
    throw std::runtime_error("interference query references a missing body");
  }
  const std::array<double, 14> values = {
      placement_a.translation[0], placement_a.translation[1],
      placement_a.translation[2], placement_a.rotation[0],
      placement_a.rotation[1],    placement_a.rotation[2],
      placement_a.rotation[3],    placement_b.translation[0],
      placement_b.translation[1], placement_b.translation[2],
      placement_b.rotation[0],    placement_b.rotation[1],
      placement_b.rotation[2],    placement_b.rotation[3]};
  if (std::any_of(values.begin(), values.end(),
                  [](double value) { return !std::isfinite(value); })) {
    throw std::runtime_error("interference query transform is not finite");
  }
  auto placed = [](const TopoDS_Shape& shape, double tx, double ty, double tz,
                   double qx, double qy, double qz, double qw) {
    const double magnitude = std::sqrt(qx * qx + qy * qy + qz * qz + qw * qw);
    if (magnitude <= 1.0e-12) {
      throw std::runtime_error("interference query quaternion is degenerate");
    }
    gp_Trsf transform;
    transform.SetRotation(gp_Quaternion(qx / magnitude, qy / magnitude,
                                        qz / magnitude, qw / magnitude));
    transform.SetTranslationPart(gp_Vec(tx, ty, tz));
    return BRepBuilderAPI_Transform(shape, transform, true).Shape();
  };
  const TopoDS_Shape a =
      placed(found_a->second, placement_a.translation[0],
             placement_a.translation[1], placement_a.translation[2],
             placement_a.rotation[0], placement_a.rotation[1],
             placement_a.rotation[2], placement_a.rotation[3]);
  const TopoDS_Shape b =
      placed(found_b->second, placement_b.translation[0],
             placement_b.translation[1], placement_b.translation[2],
             placement_b.rotation[0], placement_b.rotation[1],
             placement_b.rotation[2], placement_b.rotation[3]);

  BRepExtrema_DistShapeShape distance(a, b);
  if (!distance.IsDone()) {
    throw std::runtime_error("OCCT could not evaluate exact body clearance");
  }
  FfiInterferenceResult output{};
  output.minimum_clearance_mm = distance.Value();
  if (distance.NbSolution() > 0) {
    const gp_Pnt point_a = distance.PointOnShape1(1);
    const gp_Pnt point_b = distance.PointOnShape2(1);
    output.closest_point_a_x = point_a.X();
    output.closest_point_a_y = point_a.Y();
    output.closest_point_a_z = point_a.Z();
    output.closest_point_b_x = point_b.X();
    output.closest_point_b_y = point_b.Y();
    output.closest_point_b_z = point_b.Z();
  }

  if (!distance.InnerSolution() && output.minimum_clearance_mm > 1.0e-7) {
    return output;
  }
  BRepAlgoAPI_Common common(a, b, Message_ProgressRange());
  if (!common.IsDone()) {
    throw std::runtime_error("OCCT could not evaluate exact body overlap");
  }
  const TopoDS_Shape overlap = common.Shape();
  if (!overlap.IsNull()) {
    GProp_GProps properties;
    BRepGProp::VolumeProperties(overlap, properties);
    output.overlap_volume_mm3 = std::abs(properties.Mass());
  }
  return output;
}

FfiDrawingProjection Kernel::drawing_projection(
    const rust::Vec<std::uint64_t>& requested_body_ids,
    const rust::Vec<FfiBodyPlacement>& occurrences,
    const FfiDrawingOptions& options) const {
  if (impl_->bodies.empty()) {
    throw std::runtime_error("there are no active bodies to project");
  }
  gp_Vec direction(options.direction[0], options.direction[1],
                   options.direction[2]);
  gp_Vec up(options.up[0], options.up[1], options.up[2]);
  if (direction.SquareMagnitude() < 1.0e-18 || up.SquareMagnitude() < 1.0e-18) {
    throw std::runtime_error("drawing projection basis is degenerate");
  }
  direction.Normalize();

  gp_Vec right = up.Crossed(direction);
  if (right.SquareMagnitude() < 1.0e-18) {
    throw std::runtime_error(
        "drawing projection direction and up are parallel");
  }
  right.Normalize();

  std::vector<TopoDS_Shape> source_shapes;
  if (options.assembly_scope) {
    if (occurrences.empty()) {
      throw std::runtime_error("assembly drawing contains no occurrences");
    }
    for (const auto& occurrence : occurrences) {
      const auto found = impl_->bodies.find(occurrence.body_id);
      if (found == impl_->bodies.end()) {
        throw std::runtime_error(
            "drawing occurrence references a missing body");
      }
      const auto& t = occurrence.translation;
      const auto& q = occurrence.rotation;
      const double magnitude =
          std::sqrt(q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]);
      if (!std::isfinite(magnitude) || magnitude <= 1.0e-12 ||
          !std::isfinite(t[0]) || !std::isfinite(t[1]) ||
          !std::isfinite(t[2])) {
        throw std::runtime_error(
            "drawing occurrence placement is not a finite rigid transform");
      }
      gp_Trsf transform;
      transform.SetRotation(gp_Quaternion(q[0] / magnitude, q[1] / magnitude,
                                          q[2] / magnitude, q[3] / magnitude));
      transform.SetTranslationPart(gp_Vec(t[0], t[1], t[2]));
      source_shapes.push_back(
          BRepBuilderAPI_Transform(found->second, transform, true).Shape());
    }
  } else if (requested_body_ids.empty()) {
    for (const auto& [body_id, shape] : impl_->bodies) {
      (void)body_id;
      source_shapes.push_back(shape);
    }
  } else {
    std::set<std::uint64_t> unique_ids;
    for (const std::uint64_t body_id : requested_body_ids) {
      if (!unique_ids.insert(body_id).second) {
        continue;
      }
      const auto found = impl_->bodies.find(body_id);
      if (found == impl_->bodies.end()) {
        throw std::runtime_error("selected drawing body is missing");
      }
      source_shapes.push_back(found->second);
    }
  }

  gp_Vec section_normal(options.section_normal[0], options.section_normal[1],
                        options.section_normal[2]);
  const gp_Pnt section_point(options.section_point[0], options.section_point[1],
                             options.section_point[2]);
  if (options.has_section_plane) {
    if (section_normal.SquareMagnitude() < 1.0e-18) {
      throw std::runtime_error("drawing section plane normal is degenerate");
    }
    section_normal.Normalize();
    if (options.has_section_depth && (!std::isfinite(options.section_depth) ||
                                      options.section_depth <= 0.0)) {
      throw std::runtime_error("drawing section depth must be positive");
    }
  }

  std::vector<TopoDS_Shape> projection_shapes;
  projection_shapes.reserve(source_shapes.size());
  for (const TopoDS_Shape& source : source_shapes) {
    if (!options.has_section_plane) {
      projection_shapes.push_back(source);
      continue;
    }
    const gp_Pln front_plane(section_point, gp_Dir(section_normal));
    const gp_Pnt behind_front =
        section_point.Translated(section_normal.Multiplied(-1.0));
    TopoDS_Shape clipped = retain_half_space(source, front_plane, behind_front);
    if (options.has_section_depth && !clipped.IsNull()) {
      const gp_Pnt back_point = section_point.Translated(
          section_normal.Multiplied(-options.section_depth));
      const gp_Pln back_plane(back_point, gp_Dir(section_normal));
      const gp_Pnt inside_slab = section_point.Translated(
          section_normal.Multiplied(-options.section_depth * 0.5));
      clipped = retain_half_space(clipped, back_plane, inside_slab);
    }
    if (!clipped.IsNull()) {
      projection_shapes.push_back(clipped);
    }
  }

  Handle(HLRBRep_Algo) algorithm = new HLRBRep_Algo();
  for (const TopoDS_Shape& shape : projection_shapes) {
    algorithm->Add(shape);
  }
  algorithm->Projector(HLRAlgo_Projector(
      gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(direction), gp_Dir(right))));
  algorithm->Update();
  algorithm->Hide();

  HLRBRep_HLRToShape extractor(algorithm);
  FfiDrawingProjection output;
  output.visible_offsets.push_back(0);
  output.hidden_offsets.push_back(0);
  output.section_offsets.push_back(0);
  std::set<std::vector<std::int64_t>> seen;
  const double curve_deflection = std::max(1.0e-4, options.deflection);
  append_projection_shape(extractor.VCompound(), curve_deflection,
                          output.visible_offsets, output.visible_points, seen);
  append_projection_shape(extractor.OutLineVCompound(), curve_deflection,
                          output.visible_offsets, output.visible_points, seen);
  if (options.include_tangent_edges) {
    append_projection_shape(extractor.Rg1LineVCompound(), curve_deflection,
                            output.visible_offsets, output.visible_points,
                            seen);
    append_projection_shape(extractor.RgNLineVCompound(), curve_deflection,
                            output.visible_offsets, output.visible_points,
                            seen);
  }
  if (options.include_hidden) {
    append_projection_shape(extractor.HCompound(), curve_deflection,
                            output.hidden_offsets, output.hidden_points, seen);
    append_projection_shape(extractor.OutLineHCompound(), curve_deflection,
                            output.hidden_offsets, output.hidden_points, seen);
    if (options.include_tangent_edges) {
      append_projection_shape(extractor.Rg1LineHCompound(), curve_deflection,
                              output.hidden_offsets, output.hidden_points,
                              seen);
      append_projection_shape(extractor.RgNLineHCompound(), curve_deflection,
                              output.hidden_offsets, output.hidden_points,
                              seen);
    }
  }
  if (options.has_section_plane) {
    const gp_Pln cutting_plane(section_point, gp_Dir(section_normal));
    gp_Vec page_up = direction.Crossed(right);
    page_up.Normalize();
    std::set<std::vector<std::int64_t>> section_seen;
    for (const TopoDS_Shape& shape : source_shapes) {
      append_section_shape(exact_section_shape(shape, cutting_plane), right, page_up,
                           curve_deflection, output.section_offsets,
                           output.section_points, section_seen);
    }
  }
  return output;
}

rust::Vec<std::uint8_t> Kernel::export_step(
    const rust::Vec<std::uint64_t>& requested_body_ids,
    rust::Str thread_metadata_hex,
    rust::Str occurrence_placements_hex) const {
  if (impl_->bodies.empty()) {
    throw std::runtime_error("there are no active bodies to export");
  }
  STEPControl_Writer writer;
  if (!Interface_Static::SetIVal("write.step.schema", 5)) {
    throw std::runtime_error("OCCT does not expose the AP242 STEP schema");
  }



  (void)writer.Model(Standard_True);
  auto transfer = [&](const TopoDS_Shape& shape) {
    const IFSelect_ReturnStatus status =
        writer.Transfer(shape, STEPControl_AsIs, true, Message_ProgressRange());
    if (status != IFSelect_RetDone) {
      throw std::runtime_error("OCCT could not transfer a body to STEP");
    }
  };
  constexpr std::size_t kOccurrenceRecordBytes = 3 * sizeof(std::uint64_t) +
                                                  7 * sizeof(double);
  auto decode_hex = [](rust::Str input) {
    if (input.size() % 2 != 0) {
      throw std::runtime_error("STEP occurrence placement payload has odd hex length");
    }
    auto nibble = [](char value) -> std::uint8_t {
      if (value >= '0' && value <= '9') return value - '0';
      if (value >= 'a' && value <= 'f') return value - 'a' + 10;
      if (value >= 'A' && value <= 'F') return value - 'A' + 10;
      throw std::runtime_error("STEP occurrence placement payload is not hexadecimal");
    };
    std::vector<std::uint8_t> bytes;
    bytes.reserve(input.size() / 2);
    const char* data = input.data();
    for (std::size_t index = 0; index < input.size(); index += 2) {
      bytes.push_back(static_cast<std::uint8_t>(
          (nibble(data[index]) << 4) | nibble(data[index + 1])));
    }
    return bytes;
  };
  const auto placement_bytes = decode_hex(occurrence_placements_hex);
  if (placement_bytes.size() % kOccurrenceRecordBytes != 0) {
    throw std::runtime_error("STEP occurrence placement payload is truncated");
  }
  auto read_u64 = [&](std::size_t offset) {
    std::uint64_t value = 0;
    for (std::size_t index = 0; index < sizeof(value); ++index) {
      value |= static_cast<std::uint64_t>(placement_bytes[offset + index]) << (index * 8);
    }
    return value;
  };
  auto read_f64 = [&](std::size_t offset) {
    const std::uint64_t bits = read_u64(offset);
    double value = 0.0;
    static_assert(sizeof(value) == sizeof(bits));
    std::memcpy(&value, &bits, sizeof(value));
    return value;
  };

  if (!placement_bytes.empty()) {
    for (std::size_t offset = 0; offset < placement_bytes.size();
         offset += kOccurrenceRecordBytes) {
      const std::uint64_t body_id = read_u64(offset);



      (void)read_u64(offset + 8);
      (void)read_u64(offset + 16);
      const double tx = read_f64(offset + 24);
      const double ty = read_f64(offset + 32);
      const double tz = read_f64(offset + 40);
      const double qx = read_f64(offset + 48);
      const double qy = read_f64(offset + 56);
      const double qz = read_f64(offset + 64);
      const double qw = read_f64(offset + 72);
      const auto found = impl_->bodies.find(body_id);
      if (found == impl_->bodies.end()) {
        throw std::runtime_error("assembly STEP occurrence references a missing body");
      }
      const double magnitude = std::sqrt(qx * qx + qy * qy + qz * qz + qw * qw);
      if (magnitude <= 1.0e-12 || !std::isfinite(magnitude)) {
        throw std::runtime_error("assembly STEP occurrence rotation is degenerate");
      }
      gp_Trsf transform;
      transform.SetRotation(gp_Quaternion(
          qx / magnitude, qy / magnitude, qz / magnitude, qw / magnitude));
      transform.SetTranslationPart(gp_Vec(tx, ty, tz));
      transfer(BRepBuilderAPI_Transform(found->second, transform, true).Shape());
    }
  } else if (requested_body_ids.empty()) {
    for (const auto& [body_id, shape] : impl_->bodies) {
      (void)body_id;
      transfer(shape);
    }
  } else {
    for (const std::uint64_t body_id : requested_body_ids) {
      const auto found = impl_->bodies.find(body_id);
      if (found == impl_->bodies.end()) {
        throw std::runtime_error("selected STEP export body is missing");
      }
      transfer(found->second);
    }
  }

  if (thread_metadata_hex.size() > 4) {
    const std::string metadata(thread_metadata_hex.data(),
                               thread_metadata_hex.size());
    const std::string description =
        "Limo CAD AP242; LIMO_CAD_THREAD_METADATA_V1_HEX=" + metadata;
    const Handle(StepData_StepModel) model = writer.Model(Standard_False);
    APIHeaderSection_MakeHeader header(model);
    Handle(Interface_HArray1OfHAsciiString) descriptions =
        new Interface_HArray1OfHAsciiString(1, 1);
    descriptions->SetValue(
        1, new TCollection_HAsciiString(description.c_str()));
    header.SetDescription(descriptions);
    header.Apply(model);
  }

  std::ostringstream stream;
  if (writer.WriteStream(stream) != IFSelect_RetDone) {
    throw std::runtime_error("OCCT could not write the STEP stream");
  }
  const std::string bytes = stream.str();
  rust::Vec<std::uint8_t> output;
  output.reserve(bytes.size());
  for (const unsigned char byte : bytes) {
    output.push_back(byte);
  }
  return output;
}

std::unique_ptr<Kernel> new_kernel() {



  static const bool globals_initialized = [] {
    auto messenger = Message::DefaultMessenger();
    messenger->RemovePrinters(STANDARD_TYPE(Message_PrinterOStream));
    Handle(Message_PrinterOStream) printer =
        new Message_PrinterOStream("cerr", Standard_True);
    printer->SetToColorize(Standard_False);
    messenger->AddPrinter(printer);






    (void)BRepLib::Plane();
    return true;
  }();
  (void)globals_initialized;
  return std::make_unique<Kernel>();
}

}
