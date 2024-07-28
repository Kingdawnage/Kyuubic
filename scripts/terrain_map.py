import matplotlib.pyplot as plt
import numpy as np
from vispy import scene, app
from vispy.color import Color

# Read voxel data from file
voxel_data = []
with open("terrain_map.txt", "r") as file:
    for line in file:
        x, y, z, is_solid = line.strip().split(",")
        is_solid = is_solid.lower() == 'true'  
        voxel_data.append((int(x), int(y), int(z), is_solid))

# Separate solid and non-solid voxels
solid_voxels = [(x, y, z) for x, y, z, is_solid in voxel_data if is_solid]
non_solid_voxels = [(x, y, z) for x, y, z, is_solid in voxel_data if not is_solid]

# Convert to numpy arrays
solid_voxels = np.array(solid_voxels)

canvas = scene.SceneCanvas(keys='interactive', show=True)
view = canvas.central_widget.add_view()
view.camera = scene.TurntableCamera(up='y', fov=60)

# Add scatter plot for solid voxels
if solid_voxels.size > 0:
    scatter = scene.visuals.Markers()
    scatter.set_data(solid_voxels, face_color=Color('red'), size=5)
    view.add(scatter)

axis = scene.visuals.XYZAxis(parent=view.scene)

if __name__ == '__main__':
    app.run()


## Matplot lib plot of voxels (slow)
# # Plot solid voxels
# fig = plt.figure()
# ax = fig.add_subplot(111, projection='3d')
# if solid_voxels:
#     xs, ys, zs = zip(*solid_voxels)
#     ax.scatter(xs, ys, zs, c='r', marker='o', label='Solid')

# ax.set_xlabel('X')
# ax.set_ylabel('Y')
# ax.set_zlabel('Z')
# ax.legend()

# plt.show()
