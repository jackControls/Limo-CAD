import * as CAD from './cadInteraction';
import { ScreenLineMaterial } from './cadInteraction';
import type { OverlayLineLayer, OverlayPointLayer } from '../../cam/overlayGeometry';

/** Read the browser's actual interaction objects for headless regression checks.
 * This does not render, retain another scene, or communicate with a desktop host. */
export function inspectSketchInteraction(
  roots: Array<{ object: CAD.Object3D; lines?: boolean; points?: boolean; annotations?: boolean }>,
  marker: CAD.Object3D | null,
  worldPerPixel: number,
  camera: CAD.PerspectiveCamera,
  viewport: { width: number; height: number },
) {
  const lines = new Map<string, OverlayLineLayer>();
  const points = new Map<string, OverlayPointLayer>();
  const position = new CAD.Vector3();
  const end = new CAD.Vector3();
  const annotations: Array<{
    text: string; kind: 'constraint' | 'dimension'; screen: [number, number];
    color: [number, number, number, number]; selected: boolean; icon?: string;
  }> = [];
  const colorFor = (material: CAD.Material | null): [number, number, number, number] => {
    const color = material?.color ?? new CAD.Color(0xffffff);
    return [color.r, color.g, color.b, material?.opacity ?? 1];
  };
  for (const root of roots) {
    root.object.updateWorldMatrix(true, true);
    root.object.traverseVisible(object => {
      const label = object.userData.nativeAnnotationText;
      if (root.annotations && typeof label === 'string' && label.length) {
        const projected = object.getWorldPosition(position).clone().project(camera);
        if (projected.z >= -1 && projected.z <= 1 && Number.isFinite(projected.x) && Number.isFinite(projected.y)) {
          const colorValue = object.userData.nativeAnnotationColor;
          const opacity = object.userData.nativeAnnotationOpacity;
          const color = new CAD.Color(typeof colorValue === 'number' ? colorValue : 0xffffff);
          annotations.push({
            text: label, kind: object.userData.nativeAnnotationKind === 'constraint' ? 'constraint' : 'dimension',
            screen: [(projected.x + 1) * viewport.width / 2, (1 - projected.y) * viewport.height / 2],
            color: [color.r, color.g, color.b, typeof opacity === 'number' ? opacity : 1],
            selected: object.userData.nativeAnnotationSelected === true,
            icon: typeof object.userData.nativeConstraintIcon === 'string' ? object.userData.nativeConstraintIcon : undefined,
          });
        }
      }
      const geometry = (object as CAD.Object3D & { geometry?: CAD.BufferGeometry }).geometry;
      if (!geometry) return;
      const materials = (object as CAD.Object3D & { material?: CAD.Material | CAD.Material[] }).material;
      const material = Array.isArray(materials) ? materials[0] ?? null : materials ?? null;
      const positions = geometry.getAttribute('position');
      if (root.points && object instanceof CAD.Points && positions) {
        const colors = geometry.getAttribute('color');
        const radius = Math.max(0.08, worldPerPixel * ((material as CAD.PointsMaterial | null)?.size ?? 7) * 0.5);
        const hollow = object.userData.nativePointHollow === true;
        for (let i = 0; i < positions.count; i++) {
          position.set(positions.getX(i), positions.getY(i), positions.getZ(i)).applyMatrix4(object.matrixWorld);
          const color: [number, number, number, number] = colors
            ? [colors.getX(i), colors.getY(i), colors.getZ(i), material?.opacity ?? 1] : colorFor(material);
          const key = `${color.join(',')}|${radius}|${hollow}`;
          let layer = points.get(key);
          if (!layer) { layer = { color, radius, hollow, positions: [] }; points.set(key, layer); }
          layer.positions.push(position.x, position.y, position.z);
        }
        return;
      }
      if (!root.lines) return;
      const color = colorFor(material);
      const width = material instanceof ScreenLineMaterial ? material.linewidth : 1.25;
      const pattern = object.userData.nativeLinePattern === 'dotted' ? 'dotted' : 'solid';
      const key = `${color.join(',')}|${width}|${pattern}`;
      const append = () => {
        let layer = lines.get(key);
        if (!layer) { layer = { color, width, pattern, segments: [] }; lines.set(key, layer); }
        layer.segments.push(position.x, position.y, position.z, end.x, end.y, end.z);
      };
      const starts = geometry.getAttribute('instanceStart');
      const ends = geometry.getAttribute('instanceEnd');
      if (starts && ends) {
        for (let i = 0; i < Math.min(starts.count, ends.count); i++) {
          position.set(starts.getX(i), starts.getY(i), starts.getZ(i)).applyMatrix4(object.matrixWorld);
          end.set(ends.getX(i), ends.getY(i), ends.getZ(i)).applyMatrix4(object.matrixWorld);
          append();
        }
      } else if (positions && (object instanceof CAD.Line || object instanceof CAD.LineSegments)) {
        for (let i = 0; i + 1 < positions.count; i += object instanceof CAD.LineSegments ? 2 : 1) {
          position.set(positions.getX(i), positions.getY(i), positions.getZ(i)).applyMatrix4(object.matrixWorld);
          end.set(positions.getX(i + 1), positions.getY(i + 1), positions.getZ(i + 1)).applyMatrix4(object.matrixWorld);
          append();
        }
      } else if (positions && object instanceof CAD.Mesh) {
        for (let i = 0; i + 2 < positions.count; i += 3) {
          for (let edge = 0; edge < 3; edge++) {
            const a = i + edge, b = i + (edge + 1) % 3;
            position.set(positions.getX(a), positions.getY(a), positions.getZ(a)).applyMatrix4(object.matrixWorld);
            end.set(positions.getX(b), positions.getY(b), positions.getZ(b)).applyMatrix4(object.matrixWorld);
            append();
          }
        }
      }
    });
  }
  return {
    lines: [...lines.values()], points: [...points.values()], annotations,
    marker: marker?.visible ? {
      position: marker.getWorldPosition(position).toArray(), kind: marker.userData.snapKind as string,
    } : null,
  };
}
