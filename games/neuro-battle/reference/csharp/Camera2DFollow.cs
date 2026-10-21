// Reconstruido del asm.js (reference/il2cpp/asmjs/Camera2DFollow.js): el Camera2DFollow de los
// Standard Assets 2D, con un campo tiempoSalto agregado y sin uso. En la cámara del nivel.
using UnityEngine;

namespace UnityStandardAssets._2D
{
    public class Camera2DFollow : MonoBehaviour
    {
        public Transform target;
        public float damping = 1f;
        public float lookAheadFactor = 3f;
        public float lookAheadReturnSpeed = 0.5f;
        public float lookAheadMoveThreshold = 0.1f;

        private float m_OffsetZ;
        private Vector3 m_LastTargetPosition;
        private Vector3 m_CurrentVelocity;
        private Vector3 m_LookAheadPos;
        private float tiempoSalto;

        void Start()
        {
            m_LastTargetPosition = target.position;
            m_OffsetZ = (transform.position - target.position).z;
            transform.parent = null;
        }

        void Update()
        {
            float xMoveDelta = (target.position - m_LastTargetPosition).x;
            bool updateLookAheadTarget = Mathf.Abs(xMoveDelta) > lookAheadMoveThreshold;
            if (updateLookAheadTarget)
                m_LookAheadPos = lookAheadFactor * Vector3.right * Mathf.Sign(xMoveDelta);
            else
                m_LookAheadPos = Vector3.MoveTowards(m_LookAheadPos, Vector3.zero, Time.deltaTime * lookAheadReturnSpeed);
            Vector3 aheadTargetPos = target.position + m_LookAheadPos + Vector3.forward * m_OffsetZ;
            Vector3 newPos = Vector3.SmoothDamp(transform.position, new Vector3(aheadTargetPos.x, aheadTargetPos.y, aheadTargetPos.z),
                                                ref m_CurrentVelocity, damping);
            transform.position = newPos;
            m_LastTargetPosition = target.position;
        }
    }
}
