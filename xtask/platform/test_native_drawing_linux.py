"""Coordinate guard tests; these do not claim to exercise an X server."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("drawing_input", Path(__file__).with_name("native-drawing-linux.py"))
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)


class PublishedCoordinates(unittest.TestCase):
    def test_logical_point_maps_to_owned_physical_origin_at_both_scales(self):
        client = {"x": 0, "y": 0, "width": 1000, "height": 700}
        for scale in (1, 2):
            geometry = {"X": 30, "Y": 40, "WIDTH": 1000 * scale, "HEIGHT": 700 * scale}
            point, actual_scale = helper.physical_point(client, geometry, [321.5, 200])
            self.assertEqual(point, [30 + round(321.5 * scale), 40 + 200 * scale])
            self.assertEqual(actual_scale, [scale, scale])

    def test_rejects_nonfinite_outside_and_inconsistent_client(self):
        client = {"x": 0, "y": 0, "width": 1000, "height": 700}
        geometry = {"X": 30, "Y": 40, "WIDTH": 1000, "HEIGHT": 700}
        for point in ([float("nan"), 10], [10, float("inf")], [-1, 10], [1000, 10], [10, 700]):
            with self.assertRaises(RuntimeError):
                helper.physical_point(client, geometry, point)
        with self.assertRaises(RuntimeError):
            helper.physical_point(client, {**geometry, "WIDTH": 2000}, [10, 10])


if __name__ == "__main__":
    unittest.main()
