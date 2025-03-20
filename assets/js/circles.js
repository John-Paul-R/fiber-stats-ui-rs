
setTimeout(() => {
    // we need to wait for the svg to load, this is kind of painful...
    const plotWrapper = document.getElementById("my_plot");
    const svg = plotWrapper.firstChild;
    plotWrapper.style.display = 'flex';
    plotWrapper.style.flexDirection = 'column';
    plotWrapper.style.alignItems = 'center';

    const plotContainer = plotWrapper.appendChild(document.createElement('div'));
    plotContainer.style.position = 'relative';
    plotContainer.appendChild(svg);
    const tooltip = document.getElementById('my_plot_tooltip')
    tooltip.style.display = 'block';
    tooltip.style.position = 'absolute';
    plotContainer.appendChild(tooltip);

  const circles = document.querySelectorAll('#my_plot circle');
    console.log('value from js', circles);
    console.log('Circle count from js: ', circles.length);

    const positionedCircles = [];
    for (const circle of circles) {
        const x = circle.cx.baseVal.value;
        const y = circle.cy.baseVal.value;
        positionedCircles.push({circle, x, y})
    }
    positionedCircles.sort((a, b) => a.x - b.x);

    const kdtree = new KDTree(positionedCircles);

    // enlarge the actual circle element on hover
    let focusedPoint = null;
    /** @param svgCircle {SVGCircleElement}*/
    function focusPoint(svgCircle, x, y) {
      if (focusedPoint === svgCircle) {
        return;
      } 
      if (!!focusedPoint) {

        focusedPoint.setAttribute('transform', '');
        focusedPoint.transform = null;
      }
      svgCircle.setAttribute('transform', `scale(2,2) translate(-${x/2}, -${y/2})`);
      focusedPoint = svgCircle;
    }

    // the only 'pointer move' event handler in this system, handles everything
    plotContainer.addEventListener('pointermove', (e) => {
        const {x: plotClientX, y: plotClientY} = plotContainer.getBoundingClientRect();
        const { clientX, clientY } = e;

        const closestCircle = kdtree.findNearest(
          clientX - plotClientX,
          clientY - plotClientY);
  
        // --- Update the tooltip ---
        tooltip.style.left = closestCircle.x + 'px';
        tooltip.style.top = closestCircle.y + 'px';

        const xData = closestCircle.circle.attributes.getNamedItem('data-x').value;
        const xAsDate = new Date(xData);
        const xDataFormatted = xAsDate.getFullYear() + '-' + (xAsDate.getMonth() + 1) + '-' + xAsDate.getDate();
        const yData = closestCircle.circle.attributes.getNamedItem('data-y').value;
        tooltip.innerText = `(${xDataFormatted}, ${yData})`
        
        // --- style the point as focused (and deselect prev) ---
        focusPoint(closestCircle.circle, closestCircle.x, closestCircle.y);
    });
}, 1000);

function findClosestPointByX(array, targetX) {
  if (array.length === 0) return null;
  
  let left = 0;
  let right = array.length - 1;
  
  // If target is outside the range of the array
  if (targetX <= array[0].x) return array[0];
  if (targetX >= array[right].x) return array[right];
  
  // Binary search
  while (left <= right) {
    const mid = Math.floor((left + right) / 2);
    
    if (array[mid].x === targetX) {
      // Exact match found
      return array[mid];
    }
    
    if (array[mid].x < targetX) {
      left = mid + 1;
    } else {
      right = mid - 1;
    }
  }
  
  // At this point, left > right
  // The closest point is either at index right or left
  // Compare which one is closer to the target
  if (right < 0) return array[0];
  if (left >= array.length) return array[array.length - 1];
  
  const diffLeft = Math.abs(array[left].x - targetX);
  const diffRight = Math.abs(array[right].x - targetX);
  
  return diffLeft < diffRight ? array[left] : array[right];
}

// KD-Tree implementation for efficient 2D nearest neighbor search

class KDNode {
  constructor(point, axis, left = null, right = null) {
    this.point = point;     // The point object (with x, y properties)
    this.axis = axis;       // 0 for x-axis, 1 for y-axis
    this.left = left;       // Left child
    this.right = right;     // Right child
  }
}

class KDTree {
  constructor(points) {
    this.root = this.buildTree(points, 0);
  }
  
  // Build the KD-Tree recursively
  buildTree(points, depth) {
    if (points.length === 0) return null;
    
    const axis = depth % 2;  // Alternate between x (0) and y (1)
    
    // Sort points by the current axis
    points.sort((a, b) => {
      return axis === 0 ? a.x - b.x : a.y - b.y;
    });
    
    // Find median point
    const medianIdx = Math.floor(points.length / 2);
    const medianPoint = points[medianIdx];
    
    // Create node and construct subtrees
    const node = new KDNode(
      medianPoint,
      axis,
      this.buildTree(points.slice(0, medianIdx), depth + 1),
      this.buildTree(points.slice(medianIdx + 1), depth + 1)
    );
    
    return node;
  }
  
  // Find the nearest neighbor to the target point
  findNearest(targetX, targetY) {
    if (!this.root) return null;
    
    let best = { point: null, distance: Infinity };
    
    const search = (node, depth) => {
      if (!node) return;
      
      // Calculate distance to current node
      const dx = node.point.x - targetX;
      const dy = node.point.y - targetY;
      const distance = dx * dx + dy * dy;
      
      // Update best if current is closer
      if (distance < best.distance) {
        best.point = node.point;
        best.distance = distance;
      }
      
      // Determine which child to search first
      const axis = depth % 2;
      const value = axis === 0 ? targetX : targetY;
      const nodeValue = axis === 0 ? node.point.x : node.point.y;
      
      const nearBranch = value < nodeValue ? node.left : node.right;
      const farBranch = value < nodeValue ? node.right : node.left;
      
      // Search the near branch
      search(nearBranch, depth + 1);
      
      // Only search far branch if it could contain a closer point
      const axisDistance = Math.abs(nodeValue - value);
      if (axisDistance < best.distance) {
        search(farBranch, depth + 1);
      }
    };
    
    search(this.root, 0);
    return best.point;
  }
}

// Usage example
function findClosestPointUsing2DTree(points, targetX, targetY) {
  const kdtree = new KDTree(points);
  return kdtree.findNearest(targetX, targetY);
}
