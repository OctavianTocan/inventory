(()=>{var Ap=0,Yu=1,Cp=2;var af=1,Rp=2,bi=3,Qi=0,je=1,vn=2,Rn=0,br=1,Qn=2,Zu=3,Ju=4,Yh=5,Si=100,Pp=101,Ip=102,Fp=103,Dp=104,Zh=200,il=201,kp=202,Lp=203,ua=204,bc=205,Up=206,Np=207,Bp=208,Op=209,zp=210,Hp=211,Vp=212,Gp=213,Wp=214,Mc=0,Sc=1,wc=2,Er=3,Ec=4,Tc=5,Ac=6,Cc=7,of=0,qp=1,Xp=2,Ji=0,Jh=1,Kh=2,Qh=3,Ta=4,$p=5,jh=6,tu=7;var lf=300,Tr=301,Ar=302,Rc=303,Pc=304,sl=306,Ic=1e3,ks=1001,Fc=1002,yn=1003,Yp=1004;var Ja=1005;var Jn=1006,Ol=1007;var Ls=1008;var Ai=1009,cf=1010,hf=1011,da=1012,eu=1013,Us=1014,Kn=1015,en=1016,nu=1017,iu=1018,Cr=1020,uf=35902,df=1021,ff=1022,Hn=1023,pf=1024,mf=1025,Mr=1026,Rr=1027,su=1028,ru=1029,gf=1030,au=1031;var ou=1033,To=33776,Ao=33777,Co=33778,Ro=33779,Dc=35840,kc=35841,Lc=35842,Uc=35843,Nc=36196,Bc=37492,Oc=37496,zc=37808,Hc=37809,Vc=37810,Gc=37811,Wc=37812,qc=37813,Xc=37814,$c=37815,Yc=37816,Zc=37817,Jc=37818,Kc=37819,Qc=37820,jc=37821,Po=36492,th=36494,eh=36495,xf=36283,nh=36284,ih=36285,sh=36286;var Io=2300,rh=2301,zl=2302,Ku=2400,Qu=2401,ju=2402;var Zp=3200,Jp=3201;var vf=0,Kp=1,Yi="",Ke="srgb",Lr="srgb-linear",rl="linear",fe="srgb";var tr=7680;var td=519,Qp=512,jp=513,tm=514,yf=515,em=516,nm=517,im=518,sm=519,ah=35044;var ed="300 es",wi=2e3,Fo=2001,ji=class{addEventListener(t,e){this._listeners===void 0&&(this._listeners={});let i=this._listeners;i[t]===void 0&&(i[t]=[]),i[t].indexOf(e)===-1&&i[t].push(e)}hasEventListener(t,e){if(this._listeners===void 0)return!1;let i=this._listeners;return i[t]!==void 0&&i[t].indexOf(e)!==-1}removeEventListener(t,e){if(this._listeners===void 0)return;let s=this._listeners[t];if(s!==void 0){let r=s.indexOf(e);r!==-1&&s.splice(r,1)}}dispatchEvent(t){if(this._listeners===void 0)return;let i=this._listeners[t.type];if(i!==void 0){t.target=this;let s=i.slice(0);for(let r=0,a=s.length;r<a;r++)s[r].call(this,t);t.target=null}}},Ze=["00","01","02","03","04","05","06","07","08","09","0a","0b","0c","0d","0e","0f","10","11","12","13","14","15","16","17","18","19","1a","1b","1c","1d","1e","1f","20","21","22","23","24","25","26","27","28","29","2a","2b","2c","2d","2e","2f","30","31","32","33","34","35","36","37","38","39","3a","3b","3c","3d","3e","3f","40","41","42","43","44","45","46","47","48","49","4a","4b","4c","4d","4e","4f","50","51","52","53","54","55","56","57","58","59","5a","5b","5c","5d","5e","5f","60","61","62","63","64","65","66","67","68","69","6a","6b","6c","6d","6e","6f","70","71","72","73","74","75","76","77","78","79","7a","7b","7c","7d","7e","7f","80","81","82","83","84","85","86","87","88","89","8a","8b","8c","8d","8e","8f","90","91","92","93","94","95","96","97","98","99","9a","9b","9c","9d","9e","9f","a0","a1","a2","a3","a4","a5","a6","a7","a8","a9","aa","ab","ac","ad","ae","af","b0","b1","b2","b3","b4","b5","b6","b7","b8","b9","ba","bb","bc","bd","be","bf","c0","c1","c2","c3","c4","c5","c6","c7","c8","c9","ca","cb","cc","cd","ce","cf","d0","d1","d2","d3","d4","d5","d6","d7","d8","d9","da","db","dc","dd","de","df","e0","e1","e2","e3","e4","e5","e6","e7","e8","e9","ea","eb","ec","ed","ee","ef","f0","f1","f2","f3","f4","f5","f6","f7","f8","f9","fa","fb","fc","fd","fe","ff"];var Hl=Math.PI/180,oh=180/Math.PI;function Ei(){let n=Math.random()*4294967295|0,t=Math.random()*4294967295|0,e=Math.random()*4294967295|0,i=Math.random()*4294967295|0;return(Ze[n&255]+Ze[n>>8&255]+Ze[n>>16&255]+Ze[n>>24&255]+"-"+Ze[t&255]+Ze[t>>8&255]+"-"+Ze[t>>16&15|64]+Ze[t>>24&255]+"-"+Ze[e&63|128]+Ze[e>>8&255]+"-"+Ze[e>>16&255]+Ze[e>>24&255]+Ze[i&255]+Ze[i>>8&255]+Ze[i>>16&255]+Ze[i>>24&255]).toLowerCase()}function qe(n,t,e){return Math.max(t,Math.min(e,n))}function rm(n,t){return(n%t+t)%t}function Vl(n,t,e){return(1-e)*n+e*t}function Zn(n,t){switch(t.constructor){case Float32Array:return n;case Uint32Array:return n/4294967295;case Uint16Array:return n/65535;case Uint8Array:return n/255;case Int32Array:return Math.max(n/2147483647,-1);case Int16Array:return Math.max(n/32767,-1);case Int8Array:return Math.max(n/127,-1);default:throw new Error("Invalid component type.")}}function ge(n,t){switch(t.constructor){case Float32Array:return n;case Uint32Array:return Math.round(n*4294967295);case Uint16Array:return Math.round(n*65535);case Uint8Array:return Math.round(n*255);case Int32Array:return Math.round(n*2147483647);case Int16Array:return Math.round(n*32767);case Int8Array:return Math.round(n*127);default:throw new Error("Invalid component type.")}}var j=class n{constructor(t=0,e=0){n.prototype.isVector2=!0,this.x=t,this.y=e}get width(){return this.x}set width(t){this.x=t}get height(){return this.y}set height(t){this.y=t}set(t,e){return this.x=t,this.y=e,this}setScalar(t){return this.x=t,this.y=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y)}copy(t){return this.x=t.x,this.y=t.y,this}add(t){return this.x+=t.x,this.y+=t.y,this}addScalar(t){return this.x+=t,this.y+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this}subScalar(t){return this.x-=t,this.y-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this}multiply(t){return this.x*=t.x,this.y*=t.y,this}multiplyScalar(t){return this.x*=t,this.y*=t,this}divide(t){return this.x/=t.x,this.y/=t.y,this}divideScalar(t){return this.multiplyScalar(1/t)}applyMatrix3(t){let e=this.x,i=this.y,s=t.elements;return this.x=s[0]*e+s[3]*i+s[6],this.y=s[1]*e+s[4]*i+s[7],this}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this}clamp(t,e){return this.x=Math.max(t.x,Math.min(e.x,this.x)),this.y=Math.max(t.y,Math.min(e.y,this.y)),this}clampScalar(t,e){return this.x=Math.max(t,Math.min(e,this.x)),this.y=Math.max(t,Math.min(e,this.y)),this}clampLength(t,e){let i=this.length();return this.divideScalar(i||1).multiplyScalar(Math.max(t,Math.min(e,i)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this}negate(){return this.x=-this.x,this.y=-this.y,this}dot(t){return this.x*t.x+this.y*t.y}cross(t){return this.x*t.y-this.y*t.x}lengthSq(){return this.x*this.x+this.y*this.y}length(){return Math.sqrt(this.x*this.x+this.y*this.y)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)}normalize(){return this.divideScalar(this.length()||1)}angle(){return Math.atan2(-this.y,-this.x)+Math.PI}angleTo(t){let e=Math.sqrt(this.lengthSq()*t.lengthSq());if(e===0)return Math.PI/2;let i=this.dot(t)/e;return Math.acos(qe(i,-1,1))}distanceTo(t){return Math.sqrt(this.distanceToSquared(t))}distanceToSquared(t){let e=this.x-t.x,i=this.y-t.y;return e*e+i*i}manhattanDistanceTo(t){return Math.abs(this.x-t.x)+Math.abs(this.y-t.y)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this}lerpVectors(t,e,i){return this.x=t.x+(e.x-t.x)*i,this.y=t.y+(e.y-t.y)*i,this}equals(t){return t.x===this.x&&t.y===this.y}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this}rotateAround(t,e){let i=Math.cos(e),s=Math.sin(e),r=this.x-t.x,a=this.y-t.y;return this.x=r*i-a*s+t.x,this.y=r*s+a*i+t.y,this}random(){return this.x=Math.random(),this.y=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y}},Kt=class n{constructor(t,e,i,s,r,a,o,l,c){n.prototype.isMatrix3=!0,this.elements=[1,0,0,0,1,0,0,0,1],t!==void 0&&this.set(t,e,i,s,r,a,o,l,c)}set(t,e,i,s,r,a,o,l,c){let h=this.elements;return h[0]=t,h[1]=s,h[2]=o,h[3]=e,h[4]=r,h[5]=l,h[6]=i,h[7]=a,h[8]=c,this}identity(){return this.set(1,0,0,0,1,0,0,0,1),this}copy(t){let e=this.elements,i=t.elements;return e[0]=i[0],e[1]=i[1],e[2]=i[2],e[3]=i[3],e[4]=i[4],e[5]=i[5],e[6]=i[6],e[7]=i[7],e[8]=i[8],this}extractBasis(t,e,i){return t.setFromMatrix3Column(this,0),e.setFromMatrix3Column(this,1),i.setFromMatrix3Column(this,2),this}setFromMatrix4(t){let e=t.elements;return this.set(e[0],e[4],e[8],e[1],e[5],e[9],e[2],e[6],e[10]),this}multiply(t){return this.multiplyMatrices(this,t)}premultiply(t){return this.multiplyMatrices(t,this)}multiplyMatrices(t,e){let i=t.elements,s=e.elements,r=this.elements,a=i[0],o=i[3],l=i[6],c=i[1],h=i[4],u=i[7],d=i[2],f=i[5],g=i[8],x=s[0],m=s[3],p=s[6],T=s[1],M=s[4],y=s[7],D=s[2],S=s[5],C=s[8];return r[0]=a*x+o*T+l*D,r[3]=a*m+o*M+l*S,r[6]=a*p+o*y+l*C,r[1]=c*x+h*T+u*D,r[4]=c*m+h*M+u*S,r[7]=c*p+h*y+u*C,r[2]=d*x+f*T+g*D,r[5]=d*m+f*M+g*S,r[8]=d*p+f*y+g*C,this}multiplyScalar(t){let e=this.elements;return e[0]*=t,e[3]*=t,e[6]*=t,e[1]*=t,e[4]*=t,e[7]*=t,e[2]*=t,e[5]*=t,e[8]*=t,this}determinant(){let t=this.elements,e=t[0],i=t[1],s=t[2],r=t[3],a=t[4],o=t[5],l=t[6],c=t[7],h=t[8];return e*a*h-e*o*c-i*r*h+i*o*l+s*r*c-s*a*l}invert(){let t=this.elements,e=t[0],i=t[1],s=t[2],r=t[3],a=t[4],o=t[5],l=t[6],c=t[7],h=t[8],u=h*a-o*c,d=o*l-h*r,f=c*r-a*l,g=e*u+i*d+s*f;if(g===0)return this.set(0,0,0,0,0,0,0,0,0);let x=1/g;return t[0]=u*x,t[1]=(s*c-h*i)*x,t[2]=(o*i-s*a)*x,t[3]=d*x,t[4]=(h*e-s*l)*x,t[5]=(s*r-o*e)*x,t[6]=f*x,t[7]=(i*l-c*e)*x,t[8]=(a*e-i*r)*x,this}transpose(){let t,e=this.elements;return t=e[1],e[1]=e[3],e[3]=t,t=e[2],e[2]=e[6],e[6]=t,t=e[5],e[5]=e[7],e[7]=t,this}getNormalMatrix(t){return this.setFromMatrix4(t).invert().transpose()}transposeIntoArray(t){let e=this.elements;return t[0]=e[0],t[1]=e[3],t[2]=e[6],t[3]=e[1],t[4]=e[4],t[5]=e[7],t[6]=e[2],t[7]=e[5],t[8]=e[8],this}setUvTransform(t,e,i,s,r,a,o){let l=Math.cos(r),c=Math.sin(r);return this.set(i*l,i*c,-i*(l*a+c*o)+a+t,-s*c,s*l,-s*(-c*a+l*o)+o+e,0,0,1),this}scale(t,e){return this.premultiply(Gl.makeScale(t,e)),this}rotate(t){return this.premultiply(Gl.makeRotation(-t)),this}translate(t,e){return this.premultiply(Gl.makeTranslation(t,e)),this}makeTranslation(t,e){return t.isVector2?this.set(1,0,t.x,0,1,t.y,0,0,1):this.set(1,0,t,0,1,e,0,0,1),this}makeRotation(t){let e=Math.cos(t),i=Math.sin(t);return this.set(e,-i,0,i,e,0,0,0,1),this}makeScale(t,e){return this.set(t,0,0,0,e,0,0,0,1),this}equals(t){let e=this.elements,i=t.elements;for(let s=0;s<9;s++)if(e[s]!==i[s])return!1;return!0}fromArray(t,e=0){for(let i=0;i<9;i++)this.elements[i]=t[i+e];return this}toArray(t=[],e=0){let i=this.elements;return t[e]=i[0],t[e+1]=i[1],t[e+2]=i[2],t[e+3]=i[3],t[e+4]=i[4],t[e+5]=i[5],t[e+6]=i[6],t[e+7]=i[7],t[e+8]=i[8],t}clone(){return new this.constructor().fromArray(this.elements)}},Gl=new Kt;function _f(n){for(let t=n.length-1;t>=0;--t)if(n[t]>=65535)return!0;return!1}function fa(n){return document.createElementNS("http://www.w3.org/1999/xhtml",n)}function am(){let n=fa("canvas");return n.style.display="block",n}var nd={};function aa(n){n in nd||(nd[n]=!0,console.warn(n))}function om(n,t,e){return new Promise(function(i,s){function r(){switch(n.clientWaitSync(t,n.SYNC_FLUSH_COMMANDS_BIT,0)){case n.WAIT_FAILED:s();break;case n.TIMEOUT_EXPIRED:setTimeout(r,e);break;default:i()}}setTimeout(r,e)})}function lm(n){let t=n.elements;t[2]=.5*t[2]+.5*t[3],t[6]=.5*t[6]+.5*t[7],t[10]=.5*t[10]+.5*t[11],t[14]=.5*t[14]+.5*t[15]}function cm(n){let t=n.elements;t[11]===-1?(t[10]=-t[10]-1,t[14]=-t[14]):(t[10]=-t[10],t[14]=-t[14]+1)}var se={enabled:!0,workingColorSpace:Lr,spaces:{},convert:function(n,t,e){return this.enabled===!1||t===e||!t||!e||(this.spaces[t].transfer===fe&&(n.r=Ti(n.r),n.g=Ti(n.g),n.b=Ti(n.b)),this.spaces[t].primaries!==this.spaces[e].primaries&&(n.applyMatrix3(this.spaces[t].toXYZ),n.applyMatrix3(this.spaces[e].fromXYZ)),this.spaces[e].transfer===fe&&(n.r=Sr(n.r),n.g=Sr(n.g),n.b=Sr(n.b))),n},fromWorkingColorSpace:function(n,t){return this.convert(n,this.workingColorSpace,t)},toWorkingColorSpace:function(n,t){return this.convert(n,t,this.workingColorSpace)},getPrimaries:function(n){return this.spaces[n].primaries},getTransfer:function(n){return n===Yi?rl:this.spaces[n].transfer},getLuminanceCoefficients:function(n,t=this.workingColorSpace){return n.fromArray(this.spaces[t].luminanceCoefficients)},define:function(n){Object.assign(this.spaces,n)},_getMatrix:function(n,t,e){return n.copy(this.spaces[t].toXYZ).multiply(this.spaces[e].fromXYZ)},_getDrawingBufferColorSpace:function(n){return this.spaces[n].outputColorSpaceConfig.drawingBufferColorSpace},_getUnpackColorSpace:function(n=this.workingColorSpace){return this.spaces[n].workingColorSpaceConfig.unpackColorSpace}};function Ti(n){return n<.04045?n*.0773993808:Math.pow(n*.9478672986+.0521327014,2.4)}function Sr(n){return n<.0031308?n*12.92:1.055*Math.pow(n,.41666)-.055}var id=[.64,.33,.3,.6,.15,.06],sd=[.2126,.7152,.0722],rd=[.3127,.329],ad=new Kt().set(.4123908,.3575843,.1804808,.212639,.7151687,.0721923,.0193308,.1191948,.9505322),od=new Kt().set(3.2409699,-1.5373832,-.4986108,-.9692436,1.8759675,.0415551,.0556301,-.203977,1.0569715);se.define({[Lr]:{primaries:id,whitePoint:rd,transfer:rl,toXYZ:ad,fromXYZ:od,luminanceCoefficients:sd,workingColorSpaceConfig:{unpackColorSpace:Ke},outputColorSpaceConfig:{drawingBufferColorSpace:Ke}},[Ke]:{primaries:id,whitePoint:rd,transfer:fe,toXYZ:ad,fromXYZ:od,luminanceCoefficients:sd,outputColorSpaceConfig:{drawingBufferColorSpace:Ke}}});var er,lh=class{static getDataURL(t){if(/^data:/i.test(t.src)||typeof HTMLCanvasElement>"u")return t.src;let e;if(t instanceof HTMLCanvasElement)e=t;else{er===void 0&&(er=fa("canvas")),er.width=t.width,er.height=t.height;let i=er.getContext("2d");t instanceof ImageData?i.putImageData(t,0,0):i.drawImage(t,0,0,t.width,t.height),e=er}return e.width>2048||e.height>2048?(console.warn("THREE.ImageUtils.getDataURL: Image converted to jpg for performance reasons",t),e.toDataURL("image/jpeg",.6)):e.toDataURL("image/png")}static sRGBToLinear(t){if(typeof HTMLImageElement<"u"&&t instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&t instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&t instanceof ImageBitmap){let e=fa("canvas");e.width=t.width,e.height=t.height;let i=e.getContext("2d");i.drawImage(t,0,0,t.width,t.height);let s=i.getImageData(0,0,t.width,t.height),r=s.data;for(let a=0;a<r.length;a++)r[a]=Ti(r[a]/255)*255;return i.putImageData(s,0,0),e}else if(t.data){let e=t.data.slice(0);for(let i=0;i<e.length;i++)e instanceof Uint8Array||e instanceof Uint8ClampedArray?e[i]=Math.floor(Ti(e[i]/255)*255):e[i]=Ti(e[i]);return{data:e,width:t.width,height:t.height}}else return console.warn("THREE.ImageUtils.sRGBToLinear(): Unsupported image type. No color space conversion applied."),t}},hm=0,Do=class{constructor(t=null){this.isSource=!0,Object.defineProperty(this,"id",{value:hm++}),this.uuid=Ei(),this.data=t,this.dataReady=!0,this.version=0}set needsUpdate(t){t===!0&&this.version++}toJSON(t){let e=t===void 0||typeof t=="string";if(!e&&t.images[this.uuid]!==void 0)return t.images[this.uuid];let i={uuid:this.uuid,url:""},s=this.data;if(s!==null){let r;if(Array.isArray(s)){r=[];for(let a=0,o=s.length;a<o;a++)s[a].isDataTexture?r.push(Wl(s[a].image)):r.push(Wl(s[a]))}else r=Wl(s);i.url=r}return e||(t.images[this.uuid]=i),i}};function Wl(n){return typeof HTMLImageElement<"u"&&n instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&n instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&n instanceof ImageBitmap?lh.getDataURL(n):n.data?{data:Array.from(n.data),width:n.width,height:n.height,type:n.data.constructor.name}:(console.warn("THREE.Texture: Unable to serialize Texture."),{})}var um=0,on=class n extends ji{constructor(t=n.DEFAULT_IMAGE,e=n.DEFAULT_MAPPING,i=ks,s=ks,r=Jn,a=Ls,o=Hn,l=Ai,c=n.DEFAULT_ANISOTROPY,h=Yi){super(),this.isTexture=!0,Object.defineProperty(this,"id",{value:um++}),this.uuid=Ei(),this.name="",this.source=new Do(t),this.mipmaps=[],this.mapping=e,this.channel=0,this.wrapS=i,this.wrapT=s,this.magFilter=r,this.minFilter=a,this.anisotropy=c,this.format=o,this.internalFormat=null,this.type=l,this.offset=new j(0,0),this.repeat=new j(1,1),this.center=new j(0,0),this.rotation=0,this.matrixAutoUpdate=!0,this.matrix=new Kt,this.generateMipmaps=!0,this.premultiplyAlpha=!1,this.flipY=!0,this.unpackAlignment=4,this.colorSpace=h,this.userData={},this.version=0,this.onUpdate=null,this.isRenderTargetTexture=!1,this.pmremVersion=0}get image(){return this.source.data}set image(t=null){this.source.data=t}updateMatrix(){this.matrix.setUvTransform(this.offset.x,this.offset.y,this.repeat.x,this.repeat.y,this.rotation,this.center.x,this.center.y)}clone(){return new this.constructor().copy(this)}copy(t){return this.name=t.name,this.source=t.source,this.mipmaps=t.mipmaps.slice(0),this.mapping=t.mapping,this.channel=t.channel,this.wrapS=t.wrapS,this.wrapT=t.wrapT,this.magFilter=t.magFilter,this.minFilter=t.minFilter,this.anisotropy=t.anisotropy,this.format=t.format,this.internalFormat=t.internalFormat,this.type=t.type,this.offset.copy(t.offset),this.repeat.copy(t.repeat),this.center.copy(t.center),this.rotation=t.rotation,this.matrixAutoUpdate=t.matrixAutoUpdate,this.matrix.copy(t.matrix),this.generateMipmaps=t.generateMipmaps,this.premultiplyAlpha=t.premultiplyAlpha,this.flipY=t.flipY,this.unpackAlignment=t.unpackAlignment,this.colorSpace=t.colorSpace,this.userData=JSON.parse(JSON.stringify(t.userData)),this.needsUpdate=!0,this}toJSON(t){let e=t===void 0||typeof t=="string";if(!e&&t.textures[this.uuid]!==void 0)return t.textures[this.uuid];let i={metadata:{version:4.6,type:"Texture",generator:"Texture.toJSON"},uuid:this.uuid,name:this.name,image:this.source.toJSON(t).uuid,mapping:this.mapping,channel:this.channel,repeat:[this.repeat.x,this.repeat.y],offset:[this.offset.x,this.offset.y],center:[this.center.x,this.center.y],rotation:this.rotation,wrap:[this.wrapS,this.wrapT],format:this.format,internalFormat:this.internalFormat,type:this.type,colorSpace:this.colorSpace,minFilter:this.minFilter,magFilter:this.magFilter,anisotropy:this.anisotropy,flipY:this.flipY,generateMipmaps:this.generateMipmaps,premultiplyAlpha:this.premultiplyAlpha,unpackAlignment:this.unpackAlignment};return Object.keys(this.userData).length>0&&(i.userData=this.userData),e||(t.textures[this.uuid]=i),i}dispose(){this.dispatchEvent({type:"dispose"})}transformUv(t){if(this.mapping!==lf)return t;if(t.applyMatrix3(this.matrix),t.x<0||t.x>1)switch(this.wrapS){case Ic:t.x=t.x-Math.floor(t.x);break;case ks:t.x=t.x<0?0:1;break;case Fc:Math.abs(Math.floor(t.x)%2)===1?t.x=Math.ceil(t.x)-t.x:t.x=t.x-Math.floor(t.x);break}if(t.y<0||t.y>1)switch(this.wrapT){case Ic:t.y=t.y-Math.floor(t.y);break;case ks:t.y=t.y<0?0:1;break;case Fc:Math.abs(Math.floor(t.y)%2)===1?t.y=Math.ceil(t.y)-t.y:t.y=t.y-Math.floor(t.y);break}return this.flipY&&(t.y=1-t.y),t}set needsUpdate(t){t===!0&&(this.version++,this.source.needsUpdate=!0)}set needsPMREMUpdate(t){t===!0&&this.pmremVersion++}};on.DEFAULT_IMAGE=null;on.DEFAULT_MAPPING=lf;on.DEFAULT_ANISOTROPY=1;var xe=class n{constructor(t=0,e=0,i=0,s=1){n.prototype.isVector4=!0,this.x=t,this.y=e,this.z=i,this.w=s}get width(){return this.z}set width(t){this.z=t}get height(){return this.w}set height(t){this.w=t}set(t,e,i,s){return this.x=t,this.y=e,this.z=i,this.w=s,this}setScalar(t){return this.x=t,this.y=t,this.z=t,this.w=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setZ(t){return this.z=t,this}setW(t){return this.w=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;case 2:this.z=e;break;case 3:this.w=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;case 2:return this.z;case 3:return this.w;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y,this.z,this.w)}copy(t){return this.x=t.x,this.y=t.y,this.z=t.z,this.w=t.w!==void 0?t.w:1,this}add(t){return this.x+=t.x,this.y+=t.y,this.z+=t.z,this.w+=t.w,this}addScalar(t){return this.x+=t,this.y+=t,this.z+=t,this.w+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this.z=t.z+e.z,this.w=t.w+e.w,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this.z+=t.z*e,this.w+=t.w*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this.z-=t.z,this.w-=t.w,this}subScalar(t){return this.x-=t,this.y-=t,this.z-=t,this.w-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this.z=t.z-e.z,this.w=t.w-e.w,this}multiply(t){return this.x*=t.x,this.y*=t.y,this.z*=t.z,this.w*=t.w,this}multiplyScalar(t){return this.x*=t,this.y*=t,this.z*=t,this.w*=t,this}applyMatrix4(t){let e=this.x,i=this.y,s=this.z,r=this.w,a=t.elements;return this.x=a[0]*e+a[4]*i+a[8]*s+a[12]*r,this.y=a[1]*e+a[5]*i+a[9]*s+a[13]*r,this.z=a[2]*e+a[6]*i+a[10]*s+a[14]*r,this.w=a[3]*e+a[7]*i+a[11]*s+a[15]*r,this}divide(t){return this.x/=t.x,this.y/=t.y,this.z/=t.z,this.w/=t.w,this}divideScalar(t){return this.multiplyScalar(1/t)}setAxisAngleFromQuaternion(t){this.w=2*Math.acos(t.w);let e=Math.sqrt(1-t.w*t.w);return e<1e-4?(this.x=1,this.y=0,this.z=0):(this.x=t.x/e,this.y=t.y/e,this.z=t.z/e),this}setAxisAngleFromRotationMatrix(t){let e,i,s,r,l=t.elements,c=l[0],h=l[4],u=l[8],d=l[1],f=l[5],g=l[9],x=l[2],m=l[6],p=l[10];if(Math.abs(h-d)<.01&&Math.abs(u-x)<.01&&Math.abs(g-m)<.01){if(Math.abs(h+d)<.1&&Math.abs(u+x)<.1&&Math.abs(g+m)<.1&&Math.abs(c+f+p-3)<.1)return this.set(1,0,0,0),this;e=Math.PI;let M=(c+1)/2,y=(f+1)/2,D=(p+1)/2,S=(h+d)/4,C=(u+x)/4,P=(g+m)/4;return M>y&&M>D?M<.01?(i=0,s=.707106781,r=.707106781):(i=Math.sqrt(M),s=S/i,r=C/i):y>D?y<.01?(i=.707106781,s=0,r=.707106781):(s=Math.sqrt(y),i=S/s,r=P/s):D<.01?(i=.707106781,s=.707106781,r=0):(r=Math.sqrt(D),i=C/r,s=P/r),this.set(i,s,r,e),this}let T=Math.sqrt((m-g)*(m-g)+(u-x)*(u-x)+(d-h)*(d-h));return Math.abs(T)<.001&&(T=1),this.x=(m-g)/T,this.y=(u-x)/T,this.z=(d-h)/T,this.w=Math.acos((c+f+p-1)/2),this}setFromMatrixPosition(t){let e=t.elements;return this.x=e[12],this.y=e[13],this.z=e[14],this.w=e[15],this}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this.z=Math.min(this.z,t.z),this.w=Math.min(this.w,t.w),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this.z=Math.max(this.z,t.z),this.w=Math.max(this.w,t.w),this}clamp(t,e){return this.x=Math.max(t.x,Math.min(e.x,this.x)),this.y=Math.max(t.y,Math.min(e.y,this.y)),this.z=Math.max(t.z,Math.min(e.z,this.z)),this.w=Math.max(t.w,Math.min(e.w,this.w)),this}clampScalar(t,e){return this.x=Math.max(t,Math.min(e,this.x)),this.y=Math.max(t,Math.min(e,this.y)),this.z=Math.max(t,Math.min(e,this.z)),this.w=Math.max(t,Math.min(e,this.w)),this}clampLength(t,e){let i=this.length();return this.divideScalar(i||1).multiplyScalar(Math.max(t,Math.min(e,i)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this.w=Math.floor(this.w),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this.w=Math.ceil(this.w),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this.w=Math.round(this.w),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this.w=Math.trunc(this.w),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this.w=-this.w,this}dot(t){return this.x*t.x+this.y*t.y+this.z*t.z+this.w*t.w}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)+Math.abs(this.w)}normalize(){return this.divideScalar(this.length()||1)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this.z+=(t.z-this.z)*e,this.w+=(t.w-this.w)*e,this}lerpVectors(t,e,i){return this.x=t.x+(e.x-t.x)*i,this.y=t.y+(e.y-t.y)*i,this.z=t.z+(e.z-t.z)*i,this.w=t.w+(e.w-t.w)*i,this}equals(t){return t.x===this.x&&t.y===this.y&&t.z===this.z&&t.w===this.w}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this.z=t[e+2],this.w=t[e+3],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t[e+2]=this.z,t[e+3]=this.w,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this.z=t.getZ(e),this.w=t.getW(e),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this.w=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z,yield this.w}},ch=class extends ji{constructor(t=1,e=1,i={}){super(),this.isRenderTarget=!0,this.width=t,this.height=e,this.depth=1,this.scissor=new xe(0,0,t,e),this.scissorTest=!1,this.viewport=new xe(0,0,t,e);let s={width:t,height:e,depth:1};i=Object.assign({generateMipmaps:!1,internalFormat:null,minFilter:Jn,depthBuffer:!0,stencilBuffer:!1,resolveDepthBuffer:!0,resolveStencilBuffer:!0,depthTexture:null,samples:0,count:1},i);let r=new on(s,i.mapping,i.wrapS,i.wrapT,i.magFilter,i.minFilter,i.format,i.type,i.anisotropy,i.colorSpace);r.flipY=!1,r.generateMipmaps=i.generateMipmaps,r.internalFormat=i.internalFormat,this.textures=[];let a=i.count;for(let o=0;o<a;o++)this.textures[o]=r.clone(),this.textures[o].isRenderTargetTexture=!0;this.depthBuffer=i.depthBuffer,this.stencilBuffer=i.stencilBuffer,this.resolveDepthBuffer=i.resolveDepthBuffer,this.resolveStencilBuffer=i.resolveStencilBuffer,this.depthTexture=i.depthTexture,this.samples=i.samples}get texture(){return this.textures[0]}set texture(t){this.textures[0]=t}setSize(t,e,i=1){if(this.width!==t||this.height!==e||this.depth!==i){this.width=t,this.height=e,this.depth=i;for(let s=0,r=this.textures.length;s<r;s++)this.textures[s].image.width=t,this.textures[s].image.height=e,this.textures[s].image.depth=i;this.dispose()}this.viewport.set(0,0,t,e),this.scissor.set(0,0,t,e)}clone(){return new this.constructor().copy(this)}copy(t){this.width=t.width,this.height=t.height,this.depth=t.depth,this.scissor.copy(t.scissor),this.scissorTest=t.scissorTest,this.viewport.copy(t.viewport),this.textures.length=0;for(let i=0,s=t.textures.length;i<s;i++)this.textures[i]=t.textures[i].clone(),this.textures[i].isRenderTargetTexture=!0;let e=Object.assign({},t.texture.image);return this.texture.source=new Do(e),this.depthBuffer=t.depthBuffer,this.stencilBuffer=t.stencilBuffer,this.resolveDepthBuffer=t.resolveDepthBuffer,this.resolveStencilBuffer=t.resolveStencilBuffer,t.depthTexture!==null&&(this.depthTexture=t.depthTexture.clone()),this.samples=t.samples,this}dispose(){this.dispatchEvent({type:"dispose"})}},Ue=class extends ch{constructor(t=1,e=1,i={}){super(t,e,i),this.isWebGLRenderTarget=!0}},ko=class extends on{constructor(t=null,e=1,i=1,s=1){super(null),this.isDataArrayTexture=!0,this.image={data:t,width:e,height:i,depth:s},this.magFilter=yn,this.minFilter=yn,this.wrapR=ks,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1,this.layerUpdates=new Set}addLayerUpdate(t){this.layerUpdates.add(t)}clearLayerUpdates(){this.layerUpdates.clear()}};var hh=class extends on{constructor(t=null,e=1,i=1,s=1){super(null),this.isData3DTexture=!0,this.image={data:t,width:e,height:i,depth:s},this.magFilter=yn,this.minFilter=yn,this.wrapR=ks,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}};var ts=class{constructor(t=0,e=0,i=0,s=1){this.isQuaternion=!0,this._x=t,this._y=e,this._z=i,this._w=s}static slerpFlat(t,e,i,s,r,a,o){let l=i[s+0],c=i[s+1],h=i[s+2],u=i[s+3],d=r[a+0],f=r[a+1],g=r[a+2],x=r[a+3];if(o===0){t[e+0]=l,t[e+1]=c,t[e+2]=h,t[e+3]=u;return}if(o===1){t[e+0]=d,t[e+1]=f,t[e+2]=g,t[e+3]=x;return}if(u!==x||l!==d||c!==f||h!==g){let m=1-o,p=l*d+c*f+h*g+u*x,T=p>=0?1:-1,M=1-p*p;if(M>Number.EPSILON){let D=Math.sqrt(M),S=Math.atan2(D,p*T);m=Math.sin(m*S)/D,o=Math.sin(o*S)/D}let y=o*T;if(l=l*m+d*y,c=c*m+f*y,h=h*m+g*y,u=u*m+x*y,m===1-o){let D=1/Math.sqrt(l*l+c*c+h*h+u*u);l*=D,c*=D,h*=D,u*=D}}t[e]=l,t[e+1]=c,t[e+2]=h,t[e+3]=u}static multiplyQuaternionsFlat(t,e,i,s,r,a){let o=i[s],l=i[s+1],c=i[s+2],h=i[s+3],u=r[a],d=r[a+1],f=r[a+2],g=r[a+3];return t[e]=o*g+h*u+l*f-c*d,t[e+1]=l*g+h*d+c*u-o*f,t[e+2]=c*g+h*f+o*d-l*u,t[e+3]=h*g-o*u-l*d-c*f,t}get x(){return this._x}set x(t){this._x=t,this._onChangeCallback()}get y(){return this._y}set y(t){this._y=t,this._onChangeCallback()}get z(){return this._z}set z(t){this._z=t,this._onChangeCallback()}get w(){return this._w}set w(t){this._w=t,this._onChangeCallback()}set(t,e,i,s){return this._x=t,this._y=e,this._z=i,this._w=s,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._w)}copy(t){return this._x=t.x,this._y=t.y,this._z=t.z,this._w=t.w,this._onChangeCallback(),this}setFromEuler(t,e=!0){let i=t._x,s=t._y,r=t._z,a=t._order,o=Math.cos,l=Math.sin,c=o(i/2),h=o(s/2),u=o(r/2),d=l(i/2),f=l(s/2),g=l(r/2);switch(a){case"XYZ":this._x=d*h*u+c*f*g,this._y=c*f*u-d*h*g,this._z=c*h*g+d*f*u,this._w=c*h*u-d*f*g;break;case"YXZ":this._x=d*h*u+c*f*g,this._y=c*f*u-d*h*g,this._z=c*h*g-d*f*u,this._w=c*h*u+d*f*g;break;case"ZXY":this._x=d*h*u-c*f*g,this._y=c*f*u+d*h*g,this._z=c*h*g+d*f*u,this._w=c*h*u-d*f*g;break;case"ZYX":this._x=d*h*u-c*f*g,this._y=c*f*u+d*h*g,this._z=c*h*g-d*f*u,this._w=c*h*u+d*f*g;break;case"YZX":this._x=d*h*u+c*f*g,this._y=c*f*u+d*h*g,this._z=c*h*g-d*f*u,this._w=c*h*u-d*f*g;break;case"XZY":this._x=d*h*u-c*f*g,this._y=c*f*u-d*h*g,this._z=c*h*g+d*f*u,this._w=c*h*u+d*f*g;break;default:console.warn("THREE.Quaternion: .setFromEuler() encountered an unknown order: "+a)}return e===!0&&this._onChangeCallback(),this}setFromAxisAngle(t,e){let i=e/2,s=Math.sin(i);return this._x=t.x*s,this._y=t.y*s,this._z=t.z*s,this._w=Math.cos(i),this._onChangeCallback(),this}setFromRotationMatrix(t){let e=t.elements,i=e[0],s=e[4],r=e[8],a=e[1],o=e[5],l=e[9],c=e[2],h=e[6],u=e[10],d=i+o+u;if(d>0){let f=.5/Math.sqrt(d+1);this._w=.25/f,this._x=(h-l)*f,this._y=(r-c)*f,this._z=(a-s)*f}else if(i>o&&i>u){let f=2*Math.sqrt(1+i-o-u);this._w=(h-l)/f,this._x=.25*f,this._y=(s+a)/f,this._z=(r+c)/f}else if(o>u){let f=2*Math.sqrt(1+o-i-u);this._w=(r-c)/f,this._x=(s+a)/f,this._y=.25*f,this._z=(l+h)/f}else{let f=2*Math.sqrt(1+u-i-o);this._w=(a-s)/f,this._x=(r+c)/f,this._y=(l+h)/f,this._z=.25*f}return this._onChangeCallback(),this}setFromUnitVectors(t,e){let i=t.dot(e)+1;return i<Number.EPSILON?(i=0,Math.abs(t.x)>Math.abs(t.z)?(this._x=-t.y,this._y=t.x,this._z=0,this._w=i):(this._x=0,this._y=-t.z,this._z=t.y,this._w=i)):(this._x=t.y*e.z-t.z*e.y,this._y=t.z*e.x-t.x*e.z,this._z=t.x*e.y-t.y*e.x,this._w=i),this.normalize()}angleTo(t){return 2*Math.acos(Math.abs(qe(this.dot(t),-1,1)))}rotateTowards(t,e){let i=this.angleTo(t);if(i===0)return this;let s=Math.min(1,e/i);return this.slerp(t,s),this}identity(){return this.set(0,0,0,1)}invert(){return this.conjugate()}conjugate(){return this._x*=-1,this._y*=-1,this._z*=-1,this._onChangeCallback(),this}dot(t){return this._x*t._x+this._y*t._y+this._z*t._z+this._w*t._w}lengthSq(){return this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w}length(){return Math.sqrt(this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w)}normalize(){let t=this.length();return t===0?(this._x=0,this._y=0,this._z=0,this._w=1):(t=1/t,this._x=this._x*t,this._y=this._y*t,this._z=this._z*t,this._w=this._w*t),this._onChangeCallback(),this}multiply(t){return this.multiplyQuaternions(this,t)}premultiply(t){return this.multiplyQuaternions(t,this)}multiplyQuaternions(t,e){let i=t._x,s=t._y,r=t._z,a=t._w,o=e._x,l=e._y,c=e._z,h=e._w;return this._x=i*h+a*o+s*c-r*l,this._y=s*h+a*l+r*o-i*c,this._z=r*h+a*c+i*l-s*o,this._w=a*h-i*o-s*l-r*c,this._onChangeCallback(),this}slerp(t,e){if(e===0)return this;if(e===1)return this.copy(t);let i=this._x,s=this._y,r=this._z,a=this._w,o=a*t._w+i*t._x+s*t._y+r*t._z;if(o<0?(this._w=-t._w,this._x=-t._x,this._y=-t._y,this._z=-t._z,o=-o):this.copy(t),o>=1)return this._w=a,this._x=i,this._y=s,this._z=r,this;let l=1-o*o;if(l<=Number.EPSILON){let f=1-e;return this._w=f*a+e*this._w,this._x=f*i+e*this._x,this._y=f*s+e*this._y,this._z=f*r+e*this._z,this.normalize(),this}let c=Math.sqrt(l),h=Math.atan2(c,o),u=Math.sin((1-e)*h)/c,d=Math.sin(e*h)/c;return this._w=a*u+this._w*d,this._x=i*u+this._x*d,this._y=s*u+this._y*d,this._z=r*u+this._z*d,this._onChangeCallback(),this}slerpQuaternions(t,e,i){return this.copy(t).slerp(e,i)}random(){let t=2*Math.PI*Math.random(),e=2*Math.PI*Math.random(),i=Math.random(),s=Math.sqrt(1-i),r=Math.sqrt(i);return this.set(s*Math.sin(t),s*Math.cos(t),r*Math.sin(e),r*Math.cos(e))}equals(t){return t._x===this._x&&t._y===this._y&&t._z===this._z&&t._w===this._w}fromArray(t,e=0){return this._x=t[e],this._y=t[e+1],this._z=t[e+2],this._w=t[e+3],this._onChangeCallback(),this}toArray(t=[],e=0){return t[e]=this._x,t[e+1]=this._y,t[e+2]=this._z,t[e+3]=this._w,t}fromBufferAttribute(t,e){return this._x=t.getX(e),this._y=t.getY(e),this._z=t.getZ(e),this._w=t.getW(e),this._onChangeCallback(),this}toJSON(){return this.toArray()}_onChange(t){return this._onChangeCallback=t,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._w}},I=class n{constructor(t=0,e=0,i=0){n.prototype.isVector3=!0,this.x=t,this.y=e,this.z=i}set(t,e,i){return i===void 0&&(i=this.z),this.x=t,this.y=e,this.z=i,this}setScalar(t){return this.x=t,this.y=t,this.z=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setZ(t){return this.z=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;case 2:this.z=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;case 2:return this.z;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y,this.z)}copy(t){return this.x=t.x,this.y=t.y,this.z=t.z,this}add(t){return this.x+=t.x,this.y+=t.y,this.z+=t.z,this}addScalar(t){return this.x+=t,this.y+=t,this.z+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this.z=t.z+e.z,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this.z+=t.z*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this.z-=t.z,this}subScalar(t){return this.x-=t,this.y-=t,this.z-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this.z=t.z-e.z,this}multiply(t){return this.x*=t.x,this.y*=t.y,this.z*=t.z,this}multiplyScalar(t){return this.x*=t,this.y*=t,this.z*=t,this}multiplyVectors(t,e){return this.x=t.x*e.x,this.y=t.y*e.y,this.z=t.z*e.z,this}applyEuler(t){return this.applyQuaternion(ld.setFromEuler(t))}applyAxisAngle(t,e){return this.applyQuaternion(ld.setFromAxisAngle(t,e))}applyMatrix3(t){let e=this.x,i=this.y,s=this.z,r=t.elements;return this.x=r[0]*e+r[3]*i+r[6]*s,this.y=r[1]*e+r[4]*i+r[7]*s,this.z=r[2]*e+r[5]*i+r[8]*s,this}applyNormalMatrix(t){return this.applyMatrix3(t).normalize()}applyMatrix4(t){let e=this.x,i=this.y,s=this.z,r=t.elements,a=1/(r[3]*e+r[7]*i+r[11]*s+r[15]);return this.x=(r[0]*e+r[4]*i+r[8]*s+r[12])*a,this.y=(r[1]*e+r[5]*i+r[9]*s+r[13])*a,this.z=(r[2]*e+r[6]*i+r[10]*s+r[14])*a,this}applyQuaternion(t){let e=this.x,i=this.y,s=this.z,r=t.x,a=t.y,o=t.z,l=t.w,c=2*(a*s-o*i),h=2*(o*e-r*s),u=2*(r*i-a*e);return this.x=e+l*c+a*u-o*h,this.y=i+l*h+o*c-r*u,this.z=s+l*u+r*h-a*c,this}project(t){return this.applyMatrix4(t.matrixWorldInverse).applyMatrix4(t.projectionMatrix)}unproject(t){return this.applyMatrix4(t.projectionMatrixInverse).applyMatrix4(t.matrixWorld)}transformDirection(t){let e=this.x,i=this.y,s=this.z,r=t.elements;return this.x=r[0]*e+r[4]*i+r[8]*s,this.y=r[1]*e+r[5]*i+r[9]*s,this.z=r[2]*e+r[6]*i+r[10]*s,this.normalize()}divide(t){return this.x/=t.x,this.y/=t.y,this.z/=t.z,this}divideScalar(t){return this.multiplyScalar(1/t)}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this.z=Math.min(this.z,t.z),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this.z=Math.max(this.z,t.z),this}clamp(t,e){return this.x=Math.max(t.x,Math.min(e.x,this.x)),this.y=Math.max(t.y,Math.min(e.y,this.y)),this.z=Math.max(t.z,Math.min(e.z,this.z)),this}clampScalar(t,e){return this.x=Math.max(t,Math.min(e,this.x)),this.y=Math.max(t,Math.min(e,this.y)),this.z=Math.max(t,Math.min(e,this.z)),this}clampLength(t,e){let i=this.length();return this.divideScalar(i||1).multiplyScalar(Math.max(t,Math.min(e,i)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this}dot(t){return this.x*t.x+this.y*t.y+this.z*t.z}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)}normalize(){return this.divideScalar(this.length()||1)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this.z+=(t.z-this.z)*e,this}lerpVectors(t,e,i){return this.x=t.x+(e.x-t.x)*i,this.y=t.y+(e.y-t.y)*i,this.z=t.z+(e.z-t.z)*i,this}cross(t){return this.crossVectors(this,t)}crossVectors(t,e){let i=t.x,s=t.y,r=t.z,a=e.x,o=e.y,l=e.z;return this.x=s*l-r*o,this.y=r*a-i*l,this.z=i*o-s*a,this}projectOnVector(t){let e=t.lengthSq();if(e===0)return this.set(0,0,0);let i=t.dot(this)/e;return this.copy(t).multiplyScalar(i)}projectOnPlane(t){return ql.copy(this).projectOnVector(t),this.sub(ql)}reflect(t){return this.sub(ql.copy(t).multiplyScalar(2*this.dot(t)))}angleTo(t){let e=Math.sqrt(this.lengthSq()*t.lengthSq());if(e===0)return Math.PI/2;let i=this.dot(t)/e;return Math.acos(qe(i,-1,1))}distanceTo(t){return Math.sqrt(this.distanceToSquared(t))}distanceToSquared(t){let e=this.x-t.x,i=this.y-t.y,s=this.z-t.z;return e*e+i*i+s*s}manhattanDistanceTo(t){return Math.abs(this.x-t.x)+Math.abs(this.y-t.y)+Math.abs(this.z-t.z)}setFromSpherical(t){return this.setFromSphericalCoords(t.radius,t.phi,t.theta)}setFromSphericalCoords(t,e,i){let s=Math.sin(e)*t;return this.x=s*Math.sin(i),this.y=Math.cos(e)*t,this.z=s*Math.cos(i),this}setFromCylindrical(t){return this.setFromCylindricalCoords(t.radius,t.theta,t.y)}setFromCylindricalCoords(t,e,i){return this.x=t*Math.sin(e),this.y=i,this.z=t*Math.cos(e),this}setFromMatrixPosition(t){let e=t.elements;return this.x=e[12],this.y=e[13],this.z=e[14],this}setFromMatrixScale(t){let e=this.setFromMatrixColumn(t,0).length(),i=this.setFromMatrixColumn(t,1).length(),s=this.setFromMatrixColumn(t,2).length();return this.x=e,this.y=i,this.z=s,this}setFromMatrixColumn(t,e){return this.fromArray(t.elements,e*4)}setFromMatrix3Column(t,e){return this.fromArray(t.elements,e*3)}setFromEuler(t){return this.x=t._x,this.y=t._y,this.z=t._z,this}setFromColor(t){return this.x=t.r,this.y=t.g,this.z=t.b,this}equals(t){return t.x===this.x&&t.y===this.y&&t.z===this.z}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this.z=t[e+2],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t[e+2]=this.z,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this.z=t.getZ(e),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this}randomDirection(){let t=Math.random()*Math.PI*2,e=Math.random()*2-1,i=Math.sqrt(1-e*e);return this.x=i*Math.cos(t),this.y=e,this.z=i*Math.sin(t),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z}},ql=new I,ld=new ts,jn=class{constructor(t=new I(1/0,1/0,1/0),e=new I(-1/0,-1/0,-1/0)){this.isBox3=!0,this.min=t,this.max=e}set(t,e){return this.min.copy(t),this.max.copy(e),this}setFromArray(t){this.makeEmpty();for(let e=0,i=t.length;e<i;e+=3)this.expandByPoint(Bn.fromArray(t,e));return this}setFromBufferAttribute(t){this.makeEmpty();for(let e=0,i=t.count;e<i;e++)this.expandByPoint(Bn.fromBufferAttribute(t,e));return this}setFromPoints(t){this.makeEmpty();for(let e=0,i=t.length;e<i;e++)this.expandByPoint(t[e]);return this}setFromCenterAndSize(t,e){let i=Bn.copy(e).multiplyScalar(.5);return this.min.copy(t).sub(i),this.max.copy(t).add(i),this}setFromObject(t,e=!1){return this.makeEmpty(),this.expandByObject(t,e)}clone(){return new this.constructor().copy(this)}copy(t){return this.min.copy(t.min),this.max.copy(t.max),this}makeEmpty(){return this.min.x=this.min.y=this.min.z=1/0,this.max.x=this.max.y=this.max.z=-1/0,this}isEmpty(){return this.max.x<this.min.x||this.max.y<this.min.y||this.max.z<this.min.z}getCenter(t){return this.isEmpty()?t.set(0,0,0):t.addVectors(this.min,this.max).multiplyScalar(.5)}getSize(t){return this.isEmpty()?t.set(0,0,0):t.subVectors(this.max,this.min)}expandByPoint(t){return this.min.min(t),this.max.max(t),this}expandByVector(t){return this.min.sub(t),this.max.add(t),this}expandByScalar(t){return this.min.addScalar(-t),this.max.addScalar(t),this}expandByObject(t,e=!1){t.updateWorldMatrix(!1,!1);let i=t.geometry;if(i!==void 0){let r=i.getAttribute("position");if(e===!0&&r!==void 0&&t.isInstancedMesh!==!0)for(let a=0,o=r.count;a<o;a++)t.isMesh===!0?t.getVertexPosition(a,Bn):Bn.fromBufferAttribute(r,a),Bn.applyMatrix4(t.matrixWorld),this.expandByPoint(Bn);else t.boundingBox!==void 0?(t.boundingBox===null&&t.computeBoundingBox(),Ka.copy(t.boundingBox)):(i.boundingBox===null&&i.computeBoundingBox(),Ka.copy(i.boundingBox)),Ka.applyMatrix4(t.matrixWorld),this.union(Ka)}let s=t.children;for(let r=0,a=s.length;r<a;r++)this.expandByObject(s[r],e);return this}containsPoint(t){return t.x>=this.min.x&&t.x<=this.max.x&&t.y>=this.min.y&&t.y<=this.max.y&&t.z>=this.min.z&&t.z<=this.max.z}containsBox(t){return this.min.x<=t.min.x&&t.max.x<=this.max.x&&this.min.y<=t.min.y&&t.max.y<=this.max.y&&this.min.z<=t.min.z&&t.max.z<=this.max.z}getParameter(t,e){return e.set((t.x-this.min.x)/(this.max.x-this.min.x),(t.y-this.min.y)/(this.max.y-this.min.y),(t.z-this.min.z)/(this.max.z-this.min.z))}intersectsBox(t){return t.max.x>=this.min.x&&t.min.x<=this.max.x&&t.max.y>=this.min.y&&t.min.y<=this.max.y&&t.max.z>=this.min.z&&t.min.z<=this.max.z}intersectsSphere(t){return this.clampPoint(t.center,Bn),Bn.distanceToSquared(t.center)<=t.radius*t.radius}intersectsPlane(t){let e,i;return t.normal.x>0?(e=t.normal.x*this.min.x,i=t.normal.x*this.max.x):(e=t.normal.x*this.max.x,i=t.normal.x*this.min.x),t.normal.y>0?(e+=t.normal.y*this.min.y,i+=t.normal.y*this.max.y):(e+=t.normal.y*this.max.y,i+=t.normal.y*this.min.y),t.normal.z>0?(e+=t.normal.z*this.min.z,i+=t.normal.z*this.max.z):(e+=t.normal.z*this.max.z,i+=t.normal.z*this.min.z),e<=-t.constant&&i>=-t.constant}intersectsTriangle(t){if(this.isEmpty())return!1;this.getCenter(Jr),Qa.subVectors(this.max,Jr),nr.subVectors(t.a,Jr),ir.subVectors(t.b,Jr),sr.subVectors(t.c,Jr),Vi.subVectors(ir,nr),Gi.subVectors(sr,ir),Ts.subVectors(nr,sr);let e=[0,-Vi.z,Vi.y,0,-Gi.z,Gi.y,0,-Ts.z,Ts.y,Vi.z,0,-Vi.x,Gi.z,0,-Gi.x,Ts.z,0,-Ts.x,-Vi.y,Vi.x,0,-Gi.y,Gi.x,0,-Ts.y,Ts.x,0];return!Xl(e,nr,ir,sr,Qa)||(e=[1,0,0,0,1,0,0,0,1],!Xl(e,nr,ir,sr,Qa))?!1:(ja.crossVectors(Vi,Gi),e=[ja.x,ja.y,ja.z],Xl(e,nr,ir,sr,Qa))}clampPoint(t,e){return e.copy(t).clamp(this.min,this.max)}distanceToPoint(t){return this.clampPoint(t,Bn).distanceTo(t)}getBoundingSphere(t){return this.isEmpty()?t.makeEmpty():(this.getCenter(t.center),t.radius=this.getSize(Bn).length()*.5),t}intersect(t){return this.min.max(t.min),this.max.min(t.max),this.isEmpty()&&this.makeEmpty(),this}union(t){return this.min.min(t.min),this.max.max(t.max),this}applyMatrix4(t){return this.isEmpty()?this:(gi[0].set(this.min.x,this.min.y,this.min.z).applyMatrix4(t),gi[1].set(this.min.x,this.min.y,this.max.z).applyMatrix4(t),gi[2].set(this.min.x,this.max.y,this.min.z).applyMatrix4(t),gi[3].set(this.min.x,this.max.y,this.max.z).applyMatrix4(t),gi[4].set(this.max.x,this.min.y,this.min.z).applyMatrix4(t),gi[5].set(this.max.x,this.min.y,this.max.z).applyMatrix4(t),gi[6].set(this.max.x,this.max.y,this.min.z).applyMatrix4(t),gi[7].set(this.max.x,this.max.y,this.max.z).applyMatrix4(t),this.setFromPoints(gi),this)}translate(t){return this.min.add(t),this.max.add(t),this}equals(t){return t.min.equals(this.min)&&t.max.equals(this.max)}},gi=[new I,new I,new I,new I,new I,new I,new I,new I],Bn=new I,Ka=new jn,nr=new I,ir=new I,sr=new I,Vi=new I,Gi=new I,Ts=new I,Jr=new I,Qa=new I,ja=new I,As=new I;function Xl(n,t,e,i,s){for(let r=0,a=n.length-3;r<=a;r+=3){As.fromArray(n,r);let o=s.x*Math.abs(As.x)+s.y*Math.abs(As.y)+s.z*Math.abs(As.z),l=t.dot(As),c=e.dot(As),h=i.dot(As);if(Math.max(-Math.max(l,c,h),Math.min(l,c,h))>o)return!1}return!0}var dm=new jn,Kr=new I,$l=new I,Ns=class{constructor(t=new I,e=-1){this.isSphere=!0,this.center=t,this.radius=e}set(t,e){return this.center.copy(t),this.radius=e,this}setFromPoints(t,e){let i=this.center;e!==void 0?i.copy(e):dm.setFromPoints(t).getCenter(i);let s=0;for(let r=0,a=t.length;r<a;r++)s=Math.max(s,i.distanceToSquared(t[r]));return this.radius=Math.sqrt(s),this}copy(t){return this.center.copy(t.center),this.radius=t.radius,this}isEmpty(){return this.radius<0}makeEmpty(){return this.center.set(0,0,0),this.radius=-1,this}containsPoint(t){return t.distanceToSquared(this.center)<=this.radius*this.radius}distanceToPoint(t){return t.distanceTo(this.center)-this.radius}intersectsSphere(t){let e=this.radius+t.radius;return t.center.distanceToSquared(this.center)<=e*e}intersectsBox(t){return t.intersectsSphere(this)}intersectsPlane(t){return Math.abs(t.distanceToPoint(this.center))<=this.radius}clampPoint(t,e){let i=this.center.distanceToSquared(t);return e.copy(t),i>this.radius*this.radius&&(e.sub(this.center).normalize(),e.multiplyScalar(this.radius).add(this.center)),e}getBoundingBox(t){return this.isEmpty()?(t.makeEmpty(),t):(t.set(this.center,this.center),t.expandByScalar(this.radius),t)}applyMatrix4(t){return this.center.applyMatrix4(t),this.radius=this.radius*t.getMaxScaleOnAxis(),this}translate(t){return this.center.add(t),this}expandByPoint(t){if(this.isEmpty())return this.center.copy(t),this.radius=0,this;Kr.subVectors(t,this.center);let e=Kr.lengthSq();if(e>this.radius*this.radius){let i=Math.sqrt(e),s=(i-this.radius)*.5;this.center.addScaledVector(Kr,s/i),this.radius+=s}return this}union(t){return t.isEmpty()?this:this.isEmpty()?(this.copy(t),this):(this.center.equals(t.center)===!0?this.radius=Math.max(this.radius,t.radius):($l.subVectors(t.center,this.center).setLength(t.radius),this.expandByPoint(Kr.copy(t.center).add($l)),this.expandByPoint(Kr.copy(t.center).sub($l))),this)}equals(t){return t.center.equals(this.center)&&t.radius===this.radius}clone(){return new this.constructor().copy(this)}},xi=new I,Yl=new I,to=new I,Wi=new I,Zl=new I,eo=new I,Jl=new I,uh=class{constructor(t=new I,e=new I(0,0,-1)){this.origin=t,this.direction=e}set(t,e){return this.origin.copy(t),this.direction.copy(e),this}copy(t){return this.origin.copy(t.origin),this.direction.copy(t.direction),this}at(t,e){return e.copy(this.origin).addScaledVector(this.direction,t)}lookAt(t){return this.direction.copy(t).sub(this.origin).normalize(),this}recast(t){return this.origin.copy(this.at(t,xi)),this}closestPointToPoint(t,e){e.subVectors(t,this.origin);let i=e.dot(this.direction);return i<0?e.copy(this.origin):e.copy(this.origin).addScaledVector(this.direction,i)}distanceToPoint(t){return Math.sqrt(this.distanceSqToPoint(t))}distanceSqToPoint(t){let e=xi.subVectors(t,this.origin).dot(this.direction);return e<0?this.origin.distanceToSquared(t):(xi.copy(this.origin).addScaledVector(this.direction,e),xi.distanceToSquared(t))}distanceSqToSegment(t,e,i,s){Yl.copy(t).add(e).multiplyScalar(.5),to.copy(e).sub(t).normalize(),Wi.copy(this.origin).sub(Yl);let r=t.distanceTo(e)*.5,a=-this.direction.dot(to),o=Wi.dot(this.direction),l=-Wi.dot(to),c=Wi.lengthSq(),h=Math.abs(1-a*a),u,d,f,g;if(h>0)if(u=a*l-o,d=a*o-l,g=r*h,u>=0)if(d>=-g)if(d<=g){let x=1/h;u*=x,d*=x,f=u*(u+a*d+2*o)+d*(a*u+d+2*l)+c}else d=r,u=Math.max(0,-(a*d+o)),f=-u*u+d*(d+2*l)+c;else d=-r,u=Math.max(0,-(a*d+o)),f=-u*u+d*(d+2*l)+c;else d<=-g?(u=Math.max(0,-(-a*r+o)),d=u>0?-r:Math.min(Math.max(-r,-l),r),f=-u*u+d*(d+2*l)+c):d<=g?(u=0,d=Math.min(Math.max(-r,-l),r),f=d*(d+2*l)+c):(u=Math.max(0,-(a*r+o)),d=u>0?r:Math.min(Math.max(-r,-l),r),f=-u*u+d*(d+2*l)+c);else d=a>0?-r:r,u=Math.max(0,-(a*d+o)),f=-u*u+d*(d+2*l)+c;return i&&i.copy(this.origin).addScaledVector(this.direction,u),s&&s.copy(Yl).addScaledVector(to,d),f}intersectSphere(t,e){xi.subVectors(t.center,this.origin);let i=xi.dot(this.direction),s=xi.dot(xi)-i*i,r=t.radius*t.radius;if(s>r)return null;let a=Math.sqrt(r-s),o=i-a,l=i+a;return l<0?null:o<0?this.at(l,e):this.at(o,e)}intersectsSphere(t){return this.distanceSqToPoint(t.center)<=t.radius*t.radius}distanceToPlane(t){let e=t.normal.dot(this.direction);if(e===0)return t.distanceToPoint(this.origin)===0?0:null;let i=-(this.origin.dot(t.normal)+t.constant)/e;return i>=0?i:null}intersectPlane(t,e){let i=this.distanceToPlane(t);return i===null?null:this.at(i,e)}intersectsPlane(t){let e=t.distanceToPoint(this.origin);return e===0||t.normal.dot(this.direction)*e<0}intersectBox(t,e){let i,s,r,a,o,l,c=1/this.direction.x,h=1/this.direction.y,u=1/this.direction.z,d=this.origin;return c>=0?(i=(t.min.x-d.x)*c,s=(t.max.x-d.x)*c):(i=(t.max.x-d.x)*c,s=(t.min.x-d.x)*c),h>=0?(r=(t.min.y-d.y)*h,a=(t.max.y-d.y)*h):(r=(t.max.y-d.y)*h,a=(t.min.y-d.y)*h),i>a||r>s||((r>i||isNaN(i))&&(i=r),(a<s||isNaN(s))&&(s=a),u>=0?(o=(t.min.z-d.z)*u,l=(t.max.z-d.z)*u):(o=(t.max.z-d.z)*u,l=(t.min.z-d.z)*u),i>l||o>s)||((o>i||i!==i)&&(i=o),(l<s||s!==s)&&(s=l),s<0)?null:this.at(i>=0?i:s,e)}intersectsBox(t){return this.intersectBox(t,xi)!==null}intersectTriangle(t,e,i,s,r){Zl.subVectors(e,t),eo.subVectors(i,t),Jl.crossVectors(Zl,eo);let a=this.direction.dot(Jl),o;if(a>0){if(s)return null;o=1}else if(a<0)o=-1,a=-a;else return null;Wi.subVectors(this.origin,t);let l=o*this.direction.dot(eo.crossVectors(Wi,eo));if(l<0)return null;let c=o*this.direction.dot(Zl.cross(Wi));if(c<0||l+c>a)return null;let h=-o*Wi.dot(Jl);return h<0?null:this.at(h/a,r)}applyMatrix4(t){return this.origin.applyMatrix4(t),this.direction.transformDirection(t),this}equals(t){return t.origin.equals(this.origin)&&t.direction.equals(this.direction)}clone(){return new this.constructor().copy(this)}},pe=class n{constructor(t,e,i,s,r,a,o,l,c,h,u,d,f,g,x,m){n.prototype.isMatrix4=!0,this.elements=[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],t!==void 0&&this.set(t,e,i,s,r,a,o,l,c,h,u,d,f,g,x,m)}set(t,e,i,s,r,a,o,l,c,h,u,d,f,g,x,m){let p=this.elements;return p[0]=t,p[4]=e,p[8]=i,p[12]=s,p[1]=r,p[5]=a,p[9]=o,p[13]=l,p[2]=c,p[6]=h,p[10]=u,p[14]=d,p[3]=f,p[7]=g,p[11]=x,p[15]=m,this}identity(){return this.set(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1),this}clone(){return new n().fromArray(this.elements)}copy(t){let e=this.elements,i=t.elements;return e[0]=i[0],e[1]=i[1],e[2]=i[2],e[3]=i[3],e[4]=i[4],e[5]=i[5],e[6]=i[6],e[7]=i[7],e[8]=i[8],e[9]=i[9],e[10]=i[10],e[11]=i[11],e[12]=i[12],e[13]=i[13],e[14]=i[14],e[15]=i[15],this}copyPosition(t){let e=this.elements,i=t.elements;return e[12]=i[12],e[13]=i[13],e[14]=i[14],this}setFromMatrix3(t){let e=t.elements;return this.set(e[0],e[3],e[6],0,e[1],e[4],e[7],0,e[2],e[5],e[8],0,0,0,0,1),this}extractBasis(t,e,i){return t.setFromMatrixColumn(this,0),e.setFromMatrixColumn(this,1),i.setFromMatrixColumn(this,2),this}makeBasis(t,e,i){return this.set(t.x,e.x,i.x,0,t.y,e.y,i.y,0,t.z,e.z,i.z,0,0,0,0,1),this}extractRotation(t){let e=this.elements,i=t.elements,s=1/rr.setFromMatrixColumn(t,0).length(),r=1/rr.setFromMatrixColumn(t,1).length(),a=1/rr.setFromMatrixColumn(t,2).length();return e[0]=i[0]*s,e[1]=i[1]*s,e[2]=i[2]*s,e[3]=0,e[4]=i[4]*r,e[5]=i[5]*r,e[6]=i[6]*r,e[7]=0,e[8]=i[8]*a,e[9]=i[9]*a,e[10]=i[10]*a,e[11]=0,e[12]=0,e[13]=0,e[14]=0,e[15]=1,this}makeRotationFromEuler(t){let e=this.elements,i=t.x,s=t.y,r=t.z,a=Math.cos(i),o=Math.sin(i),l=Math.cos(s),c=Math.sin(s),h=Math.cos(r),u=Math.sin(r);if(t.order==="XYZ"){let d=a*h,f=a*u,g=o*h,x=o*u;e[0]=l*h,e[4]=-l*u,e[8]=c,e[1]=f+g*c,e[5]=d-x*c,e[9]=-o*l,e[2]=x-d*c,e[6]=g+f*c,e[10]=a*l}else if(t.order==="YXZ"){let d=l*h,f=l*u,g=c*h,x=c*u;e[0]=d+x*o,e[4]=g*o-f,e[8]=a*c,e[1]=a*u,e[5]=a*h,e[9]=-o,e[2]=f*o-g,e[6]=x+d*o,e[10]=a*l}else if(t.order==="ZXY"){let d=l*h,f=l*u,g=c*h,x=c*u;e[0]=d-x*o,e[4]=-a*u,e[8]=g+f*o,e[1]=f+g*o,e[5]=a*h,e[9]=x-d*o,e[2]=-a*c,e[6]=o,e[10]=a*l}else if(t.order==="ZYX"){let d=a*h,f=a*u,g=o*h,x=o*u;e[0]=l*h,e[4]=g*c-f,e[8]=d*c+x,e[1]=l*u,e[5]=x*c+d,e[9]=f*c-g,e[2]=-c,e[6]=o*l,e[10]=a*l}else if(t.order==="YZX"){let d=a*l,f=a*c,g=o*l,x=o*c;e[0]=l*h,e[4]=x-d*u,e[8]=g*u+f,e[1]=u,e[5]=a*h,e[9]=-o*h,e[2]=-c*h,e[6]=f*u+g,e[10]=d-x*u}else if(t.order==="XZY"){let d=a*l,f=a*c,g=o*l,x=o*c;e[0]=l*h,e[4]=-u,e[8]=c*h,e[1]=d*u+x,e[5]=a*h,e[9]=f*u-g,e[2]=g*u-f,e[6]=o*h,e[10]=x*u+d}return e[3]=0,e[7]=0,e[11]=0,e[12]=0,e[13]=0,e[14]=0,e[15]=1,this}makeRotationFromQuaternion(t){return this.compose(fm,t,pm)}lookAt(t,e,i){let s=this.elements;return gn.subVectors(t,e),gn.lengthSq()===0&&(gn.z=1),gn.normalize(),qi.crossVectors(i,gn),qi.lengthSq()===0&&(Math.abs(i.z)===1?gn.x+=1e-4:gn.z+=1e-4,gn.normalize(),qi.crossVectors(i,gn)),qi.normalize(),no.crossVectors(gn,qi),s[0]=qi.x,s[4]=no.x,s[8]=gn.x,s[1]=qi.y,s[5]=no.y,s[9]=gn.y,s[2]=qi.z,s[6]=no.z,s[10]=gn.z,this}multiply(t){return this.multiplyMatrices(this,t)}premultiply(t){return this.multiplyMatrices(t,this)}multiplyMatrices(t,e){let i=t.elements,s=e.elements,r=this.elements,a=i[0],o=i[4],l=i[8],c=i[12],h=i[1],u=i[5],d=i[9],f=i[13],g=i[2],x=i[6],m=i[10],p=i[14],T=i[3],M=i[7],y=i[11],D=i[15],S=s[0],C=s[4],P=s[8],w=s[12],_=s[1],R=s[5],G=s[9],N=s[13],z=s[2],J=s[6],V=s[10],it=s[14],W=s[3],ct=s[7],gt=s[11],Mt=s[15];return r[0]=a*S+o*_+l*z+c*W,r[4]=a*C+o*R+l*J+c*ct,r[8]=a*P+o*G+l*V+c*gt,r[12]=a*w+o*N+l*it+c*Mt,r[1]=h*S+u*_+d*z+f*W,r[5]=h*C+u*R+d*J+f*ct,r[9]=h*P+u*G+d*V+f*gt,r[13]=h*w+u*N+d*it+f*Mt,r[2]=g*S+x*_+m*z+p*W,r[6]=g*C+x*R+m*J+p*ct,r[10]=g*P+x*G+m*V+p*gt,r[14]=g*w+x*N+m*it+p*Mt,r[3]=T*S+M*_+y*z+D*W,r[7]=T*C+M*R+y*J+D*ct,r[11]=T*P+M*G+y*V+D*gt,r[15]=T*w+M*N+y*it+D*Mt,this}multiplyScalar(t){let e=this.elements;return e[0]*=t,e[4]*=t,e[8]*=t,e[12]*=t,e[1]*=t,e[5]*=t,e[9]*=t,e[13]*=t,e[2]*=t,e[6]*=t,e[10]*=t,e[14]*=t,e[3]*=t,e[7]*=t,e[11]*=t,e[15]*=t,this}determinant(){let t=this.elements,e=t[0],i=t[4],s=t[8],r=t[12],a=t[1],o=t[5],l=t[9],c=t[13],h=t[2],u=t[6],d=t[10],f=t[14],g=t[3],x=t[7],m=t[11],p=t[15];return g*(+r*l*u-s*c*u-r*o*d+i*c*d+s*o*f-i*l*f)+x*(+e*l*f-e*c*d+r*a*d-s*a*f+s*c*h-r*l*h)+m*(+e*c*u-e*o*f-r*a*u+i*a*f+r*o*h-i*c*h)+p*(-s*o*h-e*l*u+e*o*d+s*a*u-i*a*d+i*l*h)}transpose(){let t=this.elements,e;return e=t[1],t[1]=t[4],t[4]=e,e=t[2],t[2]=t[8],t[8]=e,e=t[6],t[6]=t[9],t[9]=e,e=t[3],t[3]=t[12],t[12]=e,e=t[7],t[7]=t[13],t[13]=e,e=t[11],t[11]=t[14],t[14]=e,this}setPosition(t,e,i){let s=this.elements;return t.isVector3?(s[12]=t.x,s[13]=t.y,s[14]=t.z):(s[12]=t,s[13]=e,s[14]=i),this}invert(){let t=this.elements,e=t[0],i=t[1],s=t[2],r=t[3],a=t[4],o=t[5],l=t[6],c=t[7],h=t[8],u=t[9],d=t[10],f=t[11],g=t[12],x=t[13],m=t[14],p=t[15],T=u*m*c-x*d*c+x*l*f-o*m*f-u*l*p+o*d*p,M=g*d*c-h*m*c-g*l*f+a*m*f+h*l*p-a*d*p,y=h*x*c-g*u*c+g*o*f-a*x*f-h*o*p+a*u*p,D=g*u*l-h*x*l-g*o*d+a*x*d+h*o*m-a*u*m,S=e*T+i*M+s*y+r*D;if(S===0)return this.set(0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0);let C=1/S;return t[0]=T*C,t[1]=(x*d*r-u*m*r-x*s*f+i*m*f+u*s*p-i*d*p)*C,t[2]=(o*m*r-x*l*r+x*s*c-i*m*c-o*s*p+i*l*p)*C,t[3]=(u*l*r-o*d*r-u*s*c+i*d*c+o*s*f-i*l*f)*C,t[4]=M*C,t[5]=(h*m*r-g*d*r+g*s*f-e*m*f-h*s*p+e*d*p)*C,t[6]=(g*l*r-a*m*r-g*s*c+e*m*c+a*s*p-e*l*p)*C,t[7]=(a*d*r-h*l*r+h*s*c-e*d*c-a*s*f+e*l*f)*C,t[8]=y*C,t[9]=(g*u*r-h*x*r-g*i*f+e*x*f+h*i*p-e*u*p)*C,t[10]=(a*x*r-g*o*r+g*i*c-e*x*c-a*i*p+e*o*p)*C,t[11]=(h*o*r-a*u*r-h*i*c+e*u*c+a*i*f-e*o*f)*C,t[12]=D*C,t[13]=(h*x*s-g*u*s+g*i*d-e*x*d-h*i*m+e*u*m)*C,t[14]=(g*o*s-a*x*s-g*i*l+e*x*l+a*i*m-e*o*m)*C,t[15]=(a*u*s-h*o*s+h*i*l-e*u*l-a*i*d+e*o*d)*C,this}scale(t){let e=this.elements,i=t.x,s=t.y,r=t.z;return e[0]*=i,e[4]*=s,e[8]*=r,e[1]*=i,e[5]*=s,e[9]*=r,e[2]*=i,e[6]*=s,e[10]*=r,e[3]*=i,e[7]*=s,e[11]*=r,this}getMaxScaleOnAxis(){let t=this.elements,e=t[0]*t[0]+t[1]*t[1]+t[2]*t[2],i=t[4]*t[4]+t[5]*t[5]+t[6]*t[6],s=t[8]*t[8]+t[9]*t[9]+t[10]*t[10];return Math.sqrt(Math.max(e,i,s))}makeTranslation(t,e,i){return t.isVector3?this.set(1,0,0,t.x,0,1,0,t.y,0,0,1,t.z,0,0,0,1):this.set(1,0,0,t,0,1,0,e,0,0,1,i,0,0,0,1),this}makeRotationX(t){let e=Math.cos(t),i=Math.sin(t);return this.set(1,0,0,0,0,e,-i,0,0,i,e,0,0,0,0,1),this}makeRotationY(t){let e=Math.cos(t),i=Math.sin(t);return this.set(e,0,i,0,0,1,0,0,-i,0,e,0,0,0,0,1),this}makeRotationZ(t){let e=Math.cos(t),i=Math.sin(t);return this.set(e,-i,0,0,i,e,0,0,0,0,1,0,0,0,0,1),this}makeRotationAxis(t,e){let i=Math.cos(e),s=Math.sin(e),r=1-i,a=t.x,o=t.y,l=t.z,c=r*a,h=r*o;return this.set(c*a+i,c*o-s*l,c*l+s*o,0,c*o+s*l,h*o+i,h*l-s*a,0,c*l-s*o,h*l+s*a,r*l*l+i,0,0,0,0,1),this}makeScale(t,e,i){return this.set(t,0,0,0,0,e,0,0,0,0,i,0,0,0,0,1),this}makeShear(t,e,i,s,r,a){return this.set(1,i,r,0,t,1,a,0,e,s,1,0,0,0,0,1),this}compose(t,e,i){let s=this.elements,r=e._x,a=e._y,o=e._z,l=e._w,c=r+r,h=a+a,u=o+o,d=r*c,f=r*h,g=r*u,x=a*h,m=a*u,p=o*u,T=l*c,M=l*h,y=l*u,D=i.x,S=i.y,C=i.z;return s[0]=(1-(x+p))*D,s[1]=(f+y)*D,s[2]=(g-M)*D,s[3]=0,s[4]=(f-y)*S,s[5]=(1-(d+p))*S,s[6]=(m+T)*S,s[7]=0,s[8]=(g+M)*C,s[9]=(m-T)*C,s[10]=(1-(d+x))*C,s[11]=0,s[12]=t.x,s[13]=t.y,s[14]=t.z,s[15]=1,this}decompose(t,e,i){let s=this.elements,r=rr.set(s[0],s[1],s[2]).length(),a=rr.set(s[4],s[5],s[6]).length(),o=rr.set(s[8],s[9],s[10]).length();this.determinant()<0&&(r=-r),t.x=s[12],t.y=s[13],t.z=s[14],On.copy(this);let c=1/r,h=1/a,u=1/o;return On.elements[0]*=c,On.elements[1]*=c,On.elements[2]*=c,On.elements[4]*=h,On.elements[5]*=h,On.elements[6]*=h,On.elements[8]*=u,On.elements[9]*=u,On.elements[10]*=u,e.setFromRotationMatrix(On),i.x=r,i.y=a,i.z=o,this}makePerspective(t,e,i,s,r,a,o=wi){let l=this.elements,c=2*r/(e-t),h=2*r/(i-s),u=(e+t)/(e-t),d=(i+s)/(i-s),f,g;if(o===wi)f=-(a+r)/(a-r),g=-2*a*r/(a-r);else if(o===Fo)f=-a/(a-r),g=-a*r/(a-r);else throw new Error("THREE.Matrix4.makePerspective(): Invalid coordinate system: "+o);return l[0]=c,l[4]=0,l[8]=u,l[12]=0,l[1]=0,l[5]=h,l[9]=d,l[13]=0,l[2]=0,l[6]=0,l[10]=f,l[14]=g,l[3]=0,l[7]=0,l[11]=-1,l[15]=0,this}makeOrthographic(t,e,i,s,r,a,o=wi){let l=this.elements,c=1/(e-t),h=1/(i-s),u=1/(a-r),d=(e+t)*c,f=(i+s)*h,g,x;if(o===wi)g=(a+r)*u,x=-2*u;else if(o===Fo)g=r*u,x=-1*u;else throw new Error("THREE.Matrix4.makeOrthographic(): Invalid coordinate system: "+o);return l[0]=2*c,l[4]=0,l[8]=0,l[12]=-d,l[1]=0,l[5]=2*h,l[9]=0,l[13]=-f,l[2]=0,l[6]=0,l[10]=x,l[14]=-g,l[3]=0,l[7]=0,l[11]=0,l[15]=1,this}equals(t){let e=this.elements,i=t.elements;for(let s=0;s<16;s++)if(e[s]!==i[s])return!1;return!0}fromArray(t,e=0){for(let i=0;i<16;i++)this.elements[i]=t[i+e];return this}toArray(t=[],e=0){let i=this.elements;return t[e]=i[0],t[e+1]=i[1],t[e+2]=i[2],t[e+3]=i[3],t[e+4]=i[4],t[e+5]=i[5],t[e+6]=i[6],t[e+7]=i[7],t[e+8]=i[8],t[e+9]=i[9],t[e+10]=i[10],t[e+11]=i[11],t[e+12]=i[12],t[e+13]=i[13],t[e+14]=i[14],t[e+15]=i[15],t}},rr=new I,On=new pe,fm=new I(0,0,0),pm=new I(1,1,1),qi=new I,no=new I,gn=new I,cd=new pe,hd=new ts,ti=class n{constructor(t=0,e=0,i=0,s=n.DEFAULT_ORDER){this.isEuler=!0,this._x=t,this._y=e,this._z=i,this._order=s}get x(){return this._x}set x(t){this._x=t,this._onChangeCallback()}get y(){return this._y}set y(t){this._y=t,this._onChangeCallback()}get z(){return this._z}set z(t){this._z=t,this._onChangeCallback()}get order(){return this._order}set order(t){this._order=t,this._onChangeCallback()}set(t,e,i,s=this._order){return this._x=t,this._y=e,this._z=i,this._order=s,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._order)}copy(t){return this._x=t._x,this._y=t._y,this._z=t._z,this._order=t._order,this._onChangeCallback(),this}setFromRotationMatrix(t,e=this._order,i=!0){let s=t.elements,r=s[0],a=s[4],o=s[8],l=s[1],c=s[5],h=s[9],u=s[2],d=s[6],f=s[10];switch(e){case"XYZ":this._y=Math.asin(qe(o,-1,1)),Math.abs(o)<.9999999?(this._x=Math.atan2(-h,f),this._z=Math.atan2(-a,r)):(this._x=Math.atan2(d,c),this._z=0);break;case"YXZ":this._x=Math.asin(-qe(h,-1,1)),Math.abs(h)<.9999999?(this._y=Math.atan2(o,f),this._z=Math.atan2(l,c)):(this._y=Math.atan2(-u,r),this._z=0);break;case"ZXY":this._x=Math.asin(qe(d,-1,1)),Math.abs(d)<.9999999?(this._y=Math.atan2(-u,f),this._z=Math.atan2(-a,c)):(this._y=0,this._z=Math.atan2(l,r));break;case"ZYX":this._y=Math.asin(-qe(u,-1,1)),Math.abs(u)<.9999999?(this._x=Math.atan2(d,f),this._z=Math.atan2(l,r)):(this._x=0,this._z=Math.atan2(-a,c));break;case"YZX":this._z=Math.asin(qe(l,-1,1)),Math.abs(l)<.9999999?(this._x=Math.atan2(-h,c),this._y=Math.atan2(-u,r)):(this._x=0,this._y=Math.atan2(o,f));break;case"XZY":this._z=Math.asin(-qe(a,-1,1)),Math.abs(a)<.9999999?(this._x=Math.atan2(d,c),this._y=Math.atan2(o,r)):(this._x=Math.atan2(-h,f),this._y=0);break;default:console.warn("THREE.Euler: .setFromRotationMatrix() encountered an unknown order: "+e)}return this._order=e,i===!0&&this._onChangeCallback(),this}setFromQuaternion(t,e,i){return cd.makeRotationFromQuaternion(t),this.setFromRotationMatrix(cd,e,i)}setFromVector3(t,e=this._order){return this.set(t.x,t.y,t.z,e)}reorder(t){return hd.setFromEuler(this),this.setFromQuaternion(hd,t)}equals(t){return t._x===this._x&&t._y===this._y&&t._z===this._z&&t._order===this._order}fromArray(t){return this._x=t[0],this._y=t[1],this._z=t[2],t[3]!==void 0&&(this._order=t[3]),this._onChangeCallback(),this}toArray(t=[],e=0){return t[e]=this._x,t[e+1]=this._y,t[e+2]=this._z,t[e+3]=this._order,t}_onChange(t){return this._onChangeCallback=t,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._order}};ti.DEFAULT_ORDER="XYZ";var Lo=class{constructor(){this.mask=1}set(t){this.mask=(1<<t|0)>>>0}enable(t){this.mask|=1<<t|0}enableAll(){this.mask=-1}toggle(t){this.mask^=1<<t|0}disable(t){this.mask&=~(1<<t|0)}disableAll(){this.mask=0}test(t){return(this.mask&t.mask)!==0}isEnabled(t){return(this.mask&(1<<t|0))!==0}},mm=0,ud=new I,ar=new ts,vi=new pe,io=new I,Qr=new I,gm=new I,xm=new ts,dd=new I(1,0,0),fd=new I(0,1,0),pd=new I(0,0,1),md={type:"added"},vm={type:"removed"},or={type:"childadded",child:null},Kl={type:"childremoved",child:null},ln=class n extends ji{constructor(){super(),this.isObject3D=!0,Object.defineProperty(this,"id",{value:mm++}),this.uuid=Ei(),this.name="",this.type="Object3D",this.parent=null,this.children=[],this.up=n.DEFAULT_UP.clone();let t=new I,e=new ti,i=new ts,s=new I(1,1,1);function r(){i.setFromEuler(e,!1)}function a(){e.setFromQuaternion(i,void 0,!1)}e._onChange(r),i._onChange(a),Object.defineProperties(this,{position:{configurable:!0,enumerable:!0,value:t},rotation:{configurable:!0,enumerable:!0,value:e},quaternion:{configurable:!0,enumerable:!0,value:i},scale:{configurable:!0,enumerable:!0,value:s},modelViewMatrix:{value:new pe},normalMatrix:{value:new Kt}}),this.matrix=new pe,this.matrixWorld=new pe,this.matrixAutoUpdate=n.DEFAULT_MATRIX_AUTO_UPDATE,this.matrixWorldAutoUpdate=n.DEFAULT_MATRIX_WORLD_AUTO_UPDATE,this.matrixWorldNeedsUpdate=!1,this.layers=new Lo,this.visible=!0,this.castShadow=!1,this.receiveShadow=!1,this.frustumCulled=!0,this.renderOrder=0,this.animations=[],this.userData={}}onBeforeShadow(){}onAfterShadow(){}onBeforeRender(){}onAfterRender(){}applyMatrix4(t){this.matrixAutoUpdate&&this.updateMatrix(),this.matrix.premultiply(t),this.matrix.decompose(this.position,this.quaternion,this.scale)}applyQuaternion(t){return this.quaternion.premultiply(t),this}setRotationFromAxisAngle(t,e){this.quaternion.setFromAxisAngle(t,e)}setRotationFromEuler(t){this.quaternion.setFromEuler(t,!0)}setRotationFromMatrix(t){this.quaternion.setFromRotationMatrix(t)}setRotationFromQuaternion(t){this.quaternion.copy(t)}rotateOnAxis(t,e){return ar.setFromAxisAngle(t,e),this.quaternion.multiply(ar),this}rotateOnWorldAxis(t,e){return ar.setFromAxisAngle(t,e),this.quaternion.premultiply(ar),this}rotateX(t){return this.rotateOnAxis(dd,t)}rotateY(t){return this.rotateOnAxis(fd,t)}rotateZ(t){return this.rotateOnAxis(pd,t)}translateOnAxis(t,e){return ud.copy(t).applyQuaternion(this.quaternion),this.position.add(ud.multiplyScalar(e)),this}translateX(t){return this.translateOnAxis(dd,t)}translateY(t){return this.translateOnAxis(fd,t)}translateZ(t){return this.translateOnAxis(pd,t)}localToWorld(t){return this.updateWorldMatrix(!0,!1),t.applyMatrix4(this.matrixWorld)}worldToLocal(t){return this.updateWorldMatrix(!0,!1),t.applyMatrix4(vi.copy(this.matrixWorld).invert())}lookAt(t,e,i){t.isVector3?io.copy(t):io.set(t,e,i);let s=this.parent;this.updateWorldMatrix(!0,!1),Qr.setFromMatrixPosition(this.matrixWorld),this.isCamera||this.isLight?vi.lookAt(Qr,io,this.up):vi.lookAt(io,Qr,this.up),this.quaternion.setFromRotationMatrix(vi),s&&(vi.extractRotation(s.matrixWorld),ar.setFromRotationMatrix(vi),this.quaternion.premultiply(ar.invert()))}add(t){if(arguments.length>1){for(let e=0;e<arguments.length;e++)this.add(arguments[e]);return this}return t===this?(console.error("THREE.Object3D.add: object can't be added as a child of itself.",t),this):(t&&t.isObject3D?(t.removeFromParent(),t.parent=this,this.children.push(t),t.dispatchEvent(md),or.child=t,this.dispatchEvent(or),or.child=null):console.error("THREE.Object3D.add: object not an instance of THREE.Object3D.",t),this)}remove(t){if(arguments.length>1){for(let i=0;i<arguments.length;i++)this.remove(arguments[i]);return this}let e=this.children.indexOf(t);return e!==-1&&(t.parent=null,this.children.splice(e,1),t.dispatchEvent(vm),Kl.child=t,this.dispatchEvent(Kl),Kl.child=null),this}removeFromParent(){let t=this.parent;return t!==null&&t.remove(this),this}clear(){return this.remove(...this.children)}attach(t){return this.updateWorldMatrix(!0,!1),vi.copy(this.matrixWorld).invert(),t.parent!==null&&(t.parent.updateWorldMatrix(!0,!1),vi.multiply(t.parent.matrixWorld)),t.applyMatrix4(vi),t.removeFromParent(),t.parent=this,this.children.push(t),t.updateWorldMatrix(!1,!0),t.dispatchEvent(md),or.child=t,this.dispatchEvent(or),or.child=null,this}getObjectById(t){return this.getObjectByProperty("id",t)}getObjectByName(t){return this.getObjectByProperty("name",t)}getObjectByProperty(t,e){if(this[t]===e)return this;for(let i=0,s=this.children.length;i<s;i++){let a=this.children[i].getObjectByProperty(t,e);if(a!==void 0)return a}}getObjectsByProperty(t,e,i=[]){this[t]===e&&i.push(this);let s=this.children;for(let r=0,a=s.length;r<a;r++)s[r].getObjectsByProperty(t,e,i);return i}getWorldPosition(t){return this.updateWorldMatrix(!0,!1),t.setFromMatrixPosition(this.matrixWorld)}getWorldQuaternion(t){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(Qr,t,gm),t}getWorldScale(t){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(Qr,xm,t),t}getWorldDirection(t){this.updateWorldMatrix(!0,!1);let e=this.matrixWorld.elements;return t.set(e[8],e[9],e[10]).normalize()}raycast(){}traverse(t){t(this);let e=this.children;for(let i=0,s=e.length;i<s;i++)e[i].traverse(t)}traverseVisible(t){if(this.visible===!1)return;t(this);let e=this.children;for(let i=0,s=e.length;i<s;i++)e[i].traverseVisible(t)}traverseAncestors(t){let e=this.parent;e!==null&&(t(e),e.traverseAncestors(t))}updateMatrix(){this.matrix.compose(this.position,this.quaternion,this.scale),this.matrixWorldNeedsUpdate=!0}updateMatrixWorld(t){this.matrixAutoUpdate&&this.updateMatrix(),(this.matrixWorldNeedsUpdate||t)&&(this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),this.matrixWorldNeedsUpdate=!1,t=!0);let e=this.children;for(let i=0,s=e.length;i<s;i++)e[i].updateMatrixWorld(t)}updateWorldMatrix(t,e){let i=this.parent;if(t===!0&&i!==null&&i.updateWorldMatrix(!0,!1),this.matrixAutoUpdate&&this.updateMatrix(),this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),e===!0){let s=this.children;for(let r=0,a=s.length;r<a;r++)s[r].updateWorldMatrix(!1,!0)}}toJSON(t){let e=t===void 0||typeof t=="string",i={};e&&(t={geometries:{},materials:{},textures:{},images:{},shapes:{},skeletons:{},animations:{},nodes:{}},i.metadata={version:4.6,type:"Object",generator:"Object3D.toJSON"});let s={};s.uuid=this.uuid,s.type=this.type,this.name!==""&&(s.name=this.name),this.castShadow===!0&&(s.castShadow=!0),this.receiveShadow===!0&&(s.receiveShadow=!0),this.visible===!1&&(s.visible=!1),this.frustumCulled===!1&&(s.frustumCulled=!1),this.renderOrder!==0&&(s.renderOrder=this.renderOrder),Object.keys(this.userData).length>0&&(s.userData=this.userData),s.layers=this.layers.mask,s.matrix=this.matrix.toArray(),s.up=this.up.toArray(),this.matrixAutoUpdate===!1&&(s.matrixAutoUpdate=!1),this.isInstancedMesh&&(s.type="InstancedMesh",s.count=this.count,s.instanceMatrix=this.instanceMatrix.toJSON(),this.instanceColor!==null&&(s.instanceColor=this.instanceColor.toJSON())),this.isBatchedMesh&&(s.type="BatchedMesh",s.perObjectFrustumCulled=this.perObjectFrustumCulled,s.sortObjects=this.sortObjects,s.drawRanges=this._drawRanges,s.reservedRanges=this._reservedRanges,s.visibility=this._visibility,s.active=this._active,s.bounds=this._bounds.map(o=>({boxInitialized:o.boxInitialized,boxMin:o.box.min.toArray(),boxMax:o.box.max.toArray(),sphereInitialized:o.sphereInitialized,sphereRadius:o.sphere.radius,sphereCenter:o.sphere.center.toArray()})),s.maxInstanceCount=this._maxInstanceCount,s.maxVertexCount=this._maxVertexCount,s.maxIndexCount=this._maxIndexCount,s.geometryInitialized=this._geometryInitialized,s.geometryCount=this._geometryCount,s.matricesTexture=this._matricesTexture.toJSON(t),this._colorsTexture!==null&&(s.colorsTexture=this._colorsTexture.toJSON(t)),this.boundingSphere!==null&&(s.boundingSphere={center:s.boundingSphere.center.toArray(),radius:s.boundingSphere.radius}),this.boundingBox!==null&&(s.boundingBox={min:s.boundingBox.min.toArray(),max:s.boundingBox.max.toArray()}));function r(o,l){return o[l.uuid]===void 0&&(o[l.uuid]=l.toJSON(t)),l.uuid}if(this.isScene)this.background&&(this.background.isColor?s.background=this.background.toJSON():this.background.isTexture&&(s.background=this.background.toJSON(t).uuid)),this.environment&&this.environment.isTexture&&this.environment.isRenderTargetTexture!==!0&&(s.environment=this.environment.toJSON(t).uuid);else if(this.isMesh||this.isLine||this.isPoints){s.geometry=r(t.geometries,this.geometry);let o=this.geometry.parameters;if(o!==void 0&&o.shapes!==void 0){let l=o.shapes;if(Array.isArray(l))for(let c=0,h=l.length;c<h;c++){let u=l[c];r(t.shapes,u)}else r(t.shapes,l)}}if(this.isSkinnedMesh&&(s.bindMode=this.bindMode,s.bindMatrix=this.bindMatrix.toArray(),this.skeleton!==void 0&&(r(t.skeletons,this.skeleton),s.skeleton=this.skeleton.uuid)),this.material!==void 0)if(Array.isArray(this.material)){let o=[];for(let l=0,c=this.material.length;l<c;l++)o.push(r(t.materials,this.material[l]));s.material=o}else s.material=r(t.materials,this.material);if(this.children.length>0){s.children=[];for(let o=0;o<this.children.length;o++)s.children.push(this.children[o].toJSON(t).object)}if(this.animations.length>0){s.animations=[];for(let o=0;o<this.animations.length;o++){let l=this.animations[o];s.animations.push(r(t.animations,l))}}if(e){let o=a(t.geometries),l=a(t.materials),c=a(t.textures),h=a(t.images),u=a(t.shapes),d=a(t.skeletons),f=a(t.animations),g=a(t.nodes);o.length>0&&(i.geometries=o),l.length>0&&(i.materials=l),c.length>0&&(i.textures=c),h.length>0&&(i.images=h),u.length>0&&(i.shapes=u),d.length>0&&(i.skeletons=d),f.length>0&&(i.animations=f),g.length>0&&(i.nodes=g)}return i.object=s,i;function a(o){let l=[];for(let c in o){let h=o[c];delete h.metadata,l.push(h)}return l}}clone(t){return new this.constructor().copy(this,t)}copy(t,e=!0){if(this.name=t.name,this.up.copy(t.up),this.position.copy(t.position),this.rotation.order=t.rotation.order,this.quaternion.copy(t.quaternion),this.scale.copy(t.scale),this.matrix.copy(t.matrix),this.matrixWorld.copy(t.matrixWorld),this.matrixAutoUpdate=t.matrixAutoUpdate,this.matrixWorldAutoUpdate=t.matrixWorldAutoUpdate,this.matrixWorldNeedsUpdate=t.matrixWorldNeedsUpdate,this.layers.mask=t.layers.mask,this.visible=t.visible,this.castShadow=t.castShadow,this.receiveShadow=t.receiveShadow,this.frustumCulled=t.frustumCulled,this.renderOrder=t.renderOrder,this.animations=t.animations.slice(),this.userData=JSON.parse(JSON.stringify(t.userData)),e===!0)for(let i=0;i<t.children.length;i++){let s=t.children[i];this.add(s.clone())}return this}};ln.DEFAULT_UP=new I(0,1,0);ln.DEFAULT_MATRIX_AUTO_UPDATE=!0;ln.DEFAULT_MATRIX_WORLD_AUTO_UPDATE=!0;var zn=new I,yi=new I,Ql=new I,_i=new I,lr=new I,cr=new I,gd=new I,jl=new I,tc=new I,ec=new I,nc=new xe,ic=new xe,sc=new xe,Zi=class n{constructor(t=new I,e=new I,i=new I){this.a=t,this.b=e,this.c=i}static getNormal(t,e,i,s){s.subVectors(i,e),zn.subVectors(t,e),s.cross(zn);let r=s.lengthSq();return r>0?s.multiplyScalar(1/Math.sqrt(r)):s.set(0,0,0)}static getBarycoord(t,e,i,s,r){zn.subVectors(s,e),yi.subVectors(i,e),Ql.subVectors(t,e);let a=zn.dot(zn),o=zn.dot(yi),l=zn.dot(Ql),c=yi.dot(yi),h=yi.dot(Ql),u=a*c-o*o;if(u===0)return r.set(0,0,0),null;let d=1/u,f=(c*l-o*h)*d,g=(a*h-o*l)*d;return r.set(1-f-g,g,f)}static containsPoint(t,e,i,s){return this.getBarycoord(t,e,i,s,_i)===null?!1:_i.x>=0&&_i.y>=0&&_i.x+_i.y<=1}static getInterpolation(t,e,i,s,r,a,o,l){return this.getBarycoord(t,e,i,s,_i)===null?(l.x=0,l.y=0,"z"in l&&(l.z=0),"w"in l&&(l.w=0),null):(l.setScalar(0),l.addScaledVector(r,_i.x),l.addScaledVector(a,_i.y),l.addScaledVector(o,_i.z),l)}static getInterpolatedAttribute(t,e,i,s,r,a){return nc.setScalar(0),ic.setScalar(0),sc.setScalar(0),nc.fromBufferAttribute(t,e),ic.fromBufferAttribute(t,i),sc.fromBufferAttribute(t,s),a.setScalar(0),a.addScaledVector(nc,r.x),a.addScaledVector(ic,r.y),a.addScaledVector(sc,r.z),a}static isFrontFacing(t,e,i,s){return zn.subVectors(i,e),yi.subVectors(t,e),zn.cross(yi).dot(s)<0}set(t,e,i){return this.a.copy(t),this.b.copy(e),this.c.copy(i),this}setFromPointsAndIndices(t,e,i,s){return this.a.copy(t[e]),this.b.copy(t[i]),this.c.copy(t[s]),this}setFromAttributeAndIndices(t,e,i,s){return this.a.fromBufferAttribute(t,e),this.b.fromBufferAttribute(t,i),this.c.fromBufferAttribute(t,s),this}clone(){return new this.constructor().copy(this)}copy(t){return this.a.copy(t.a),this.b.copy(t.b),this.c.copy(t.c),this}getArea(){return zn.subVectors(this.c,this.b),yi.subVectors(this.a,this.b),zn.cross(yi).length()*.5}getMidpoint(t){return t.addVectors(this.a,this.b).add(this.c).multiplyScalar(1/3)}getNormal(t){return n.getNormal(this.a,this.b,this.c,t)}getPlane(t){return t.setFromCoplanarPoints(this.a,this.b,this.c)}getBarycoord(t,e){return n.getBarycoord(t,this.a,this.b,this.c,e)}getInterpolation(t,e,i,s,r){return n.getInterpolation(t,this.a,this.b,this.c,e,i,s,r)}containsPoint(t){return n.containsPoint(t,this.a,this.b,this.c)}isFrontFacing(t){return n.isFrontFacing(this.a,this.b,this.c,t)}intersectsBox(t){return t.intersectsTriangle(this)}closestPointToPoint(t,e){let i=this.a,s=this.b,r=this.c,a,o;lr.subVectors(s,i),cr.subVectors(r,i),jl.subVectors(t,i);let l=lr.dot(jl),c=cr.dot(jl);if(l<=0&&c<=0)return e.copy(i);tc.subVectors(t,s);let h=lr.dot(tc),u=cr.dot(tc);if(h>=0&&u<=h)return e.copy(s);let d=l*u-h*c;if(d<=0&&l>=0&&h<=0)return a=l/(l-h),e.copy(i).addScaledVector(lr,a);ec.subVectors(t,r);let f=lr.dot(ec),g=cr.dot(ec);if(g>=0&&f<=g)return e.copy(r);let x=f*c-l*g;if(x<=0&&c>=0&&g<=0)return o=c/(c-g),e.copy(i).addScaledVector(cr,o);let m=h*g-f*u;if(m<=0&&u-h>=0&&f-g>=0)return gd.subVectors(r,s),o=(u-h)/(u-h+(f-g)),e.copy(s).addScaledVector(gd,o);let p=1/(m+x+d);return a=x*p,o=d*p,e.copy(i).addScaledVector(lr,a).addScaledVector(cr,o)}equals(t){return t.a.equals(this.a)&&t.b.equals(this.b)&&t.c.equals(this.c)}},bf={aliceblue:15792383,antiquewhite:16444375,aqua:65535,aquamarine:8388564,azure:15794175,beige:16119260,bisque:16770244,black:0,blanchedalmond:16772045,blue:255,blueviolet:9055202,brown:10824234,burlywood:14596231,cadetblue:6266528,chartreuse:8388352,chocolate:13789470,coral:16744272,cornflowerblue:6591981,cornsilk:16775388,crimson:14423100,cyan:65535,darkblue:139,darkcyan:35723,darkgoldenrod:12092939,darkgray:11119017,darkgreen:25600,darkgrey:11119017,darkkhaki:12433259,darkmagenta:9109643,darkolivegreen:5597999,darkorange:16747520,darkorchid:10040012,darkred:9109504,darksalmon:15308410,darkseagreen:9419919,darkslateblue:4734347,darkslategray:3100495,darkslategrey:3100495,darkturquoise:52945,darkviolet:9699539,deeppink:16716947,deepskyblue:49151,dimgray:6908265,dimgrey:6908265,dodgerblue:2003199,firebrick:11674146,floralwhite:16775920,forestgreen:2263842,fuchsia:16711935,gainsboro:14474460,ghostwhite:16316671,gold:16766720,goldenrod:14329120,gray:8421504,green:32768,greenyellow:11403055,grey:8421504,honeydew:15794160,hotpink:16738740,indianred:13458524,indigo:4915330,ivory:16777200,khaki:15787660,lavender:15132410,lavenderblush:16773365,lawngreen:8190976,lemonchiffon:16775885,lightblue:11393254,lightcoral:15761536,lightcyan:14745599,lightgoldenrodyellow:16448210,lightgray:13882323,lightgreen:9498256,lightgrey:13882323,lightpink:16758465,lightsalmon:16752762,lightseagreen:2142890,lightskyblue:8900346,lightslategray:7833753,lightslategrey:7833753,lightsteelblue:11584734,lightyellow:16777184,lime:65280,limegreen:3329330,linen:16445670,magenta:16711935,maroon:8388608,mediumaquamarine:6737322,mediumblue:205,mediumorchid:12211667,mediumpurple:9662683,mediumseagreen:3978097,mediumslateblue:8087790,mediumspringgreen:64154,mediumturquoise:4772300,mediumvioletred:13047173,midnightblue:1644912,mintcream:16121850,mistyrose:16770273,moccasin:16770229,navajowhite:16768685,navy:128,oldlace:16643558,olive:8421376,olivedrab:7048739,orange:16753920,orangered:16729344,orchid:14315734,palegoldenrod:15657130,palegreen:10025880,paleturquoise:11529966,palevioletred:14381203,papayawhip:16773077,peachpuff:16767673,peru:13468991,pink:16761035,plum:14524637,powderblue:11591910,purple:8388736,rebeccapurple:6697881,red:16711680,rosybrown:12357519,royalblue:4286945,saddlebrown:9127187,salmon:16416882,sandybrown:16032864,seagreen:3050327,seashell:16774638,sienna:10506797,silver:12632256,skyblue:8900331,slateblue:6970061,slategray:7372944,slategrey:7372944,snow:16775930,springgreen:65407,steelblue:4620980,tan:13808780,teal:32896,thistle:14204888,tomato:16737095,turquoise:4251856,violet:15631086,wheat:16113331,white:16777215,whitesmoke:16119285,yellow:16776960,yellowgreen:10145074},Xi={h:0,s:0,l:0},so={h:0,s:0,l:0};function rc(n,t,e){return e<0&&(e+=1),e>1&&(e-=1),e<1/6?n+(t-n)*6*e:e<1/2?t:e<2/3?n+(t-n)*6*(2/3-e):n}var Pt=class{constructor(t,e,i){return this.isColor=!0,this.r=1,this.g=1,this.b=1,this.set(t,e,i)}set(t,e,i){if(e===void 0&&i===void 0){let s=t;s&&s.isColor?this.copy(s):typeof s=="number"?this.setHex(s):typeof s=="string"&&this.setStyle(s)}else this.setRGB(t,e,i);return this}setScalar(t){return this.r=t,this.g=t,this.b=t,this}setHex(t,e=Ke){return t=Math.floor(t),this.r=(t>>16&255)/255,this.g=(t>>8&255)/255,this.b=(t&255)/255,se.toWorkingColorSpace(this,e),this}setRGB(t,e,i,s=se.workingColorSpace){return this.r=t,this.g=e,this.b=i,se.toWorkingColorSpace(this,s),this}setHSL(t,e,i,s=se.workingColorSpace){if(t=rm(t,1),e=qe(e,0,1),i=qe(i,0,1),e===0)this.r=this.g=this.b=i;else{let r=i<=.5?i*(1+e):i+e-i*e,a=2*i-r;this.r=rc(a,r,t+1/3),this.g=rc(a,r,t),this.b=rc(a,r,t-1/3)}return se.toWorkingColorSpace(this,s),this}setStyle(t,e=Ke){function i(r){r!==void 0&&parseFloat(r)<1&&console.warn("THREE.Color: Alpha component of "+t+" will be ignored.")}let s;if(s=/^(\w+)\(([^\)]*)\)/.exec(t)){let r,a=s[1],o=s[2];switch(a){case"rgb":case"rgba":if(r=/^\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(o))return i(r[4]),this.setRGB(Math.min(255,parseInt(r[1],10))/255,Math.min(255,parseInt(r[2],10))/255,Math.min(255,parseInt(r[3],10))/255,e);if(r=/^\s*(\d+)\%\s*,\s*(\d+)\%\s*,\s*(\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(o))return i(r[4]),this.setRGB(Math.min(100,parseInt(r[1],10))/100,Math.min(100,parseInt(r[2],10))/100,Math.min(100,parseInt(r[3],10))/100,e);break;case"hsl":case"hsla":if(r=/^\s*(\d*\.?\d+)\s*,\s*(\d*\.?\d+)\%\s*,\s*(\d*\.?\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(o))return i(r[4]),this.setHSL(parseFloat(r[1])/360,parseFloat(r[2])/100,parseFloat(r[3])/100,e);break;default:console.warn("THREE.Color: Unknown color model "+t)}}else if(s=/^\#([A-Fa-f\d]+)$/.exec(t)){let r=s[1],a=r.length;if(a===3)return this.setRGB(parseInt(r.charAt(0),16)/15,parseInt(r.charAt(1),16)/15,parseInt(r.charAt(2),16)/15,e);if(a===6)return this.setHex(parseInt(r,16),e);console.warn("THREE.Color: Invalid hex color "+t)}else if(t&&t.length>0)return this.setColorName(t,e);return this}setColorName(t,e=Ke){let i=bf[t.toLowerCase()];return i!==void 0?this.setHex(i,e):console.warn("THREE.Color: Unknown color "+t),this}clone(){return new this.constructor(this.r,this.g,this.b)}copy(t){return this.r=t.r,this.g=t.g,this.b=t.b,this}copySRGBToLinear(t){return this.r=Ti(t.r),this.g=Ti(t.g),this.b=Ti(t.b),this}copyLinearToSRGB(t){return this.r=Sr(t.r),this.g=Sr(t.g),this.b=Sr(t.b),this}convertSRGBToLinear(){return this.copySRGBToLinear(this),this}convertLinearToSRGB(){return this.copyLinearToSRGB(this),this}getHex(t=Ke){return se.fromWorkingColorSpace(Je.copy(this),t),Math.round(qe(Je.r*255,0,255))*65536+Math.round(qe(Je.g*255,0,255))*256+Math.round(qe(Je.b*255,0,255))}getHexString(t=Ke){return("000000"+this.getHex(t).toString(16)).slice(-6)}getHSL(t,e=se.workingColorSpace){se.fromWorkingColorSpace(Je.copy(this),e);let i=Je.r,s=Je.g,r=Je.b,a=Math.max(i,s,r),o=Math.min(i,s,r),l,c,h=(o+a)/2;if(o===a)l=0,c=0;else{let u=a-o;switch(c=h<=.5?u/(a+o):u/(2-a-o),a){case i:l=(s-r)/u+(s<r?6:0);break;case s:l=(r-i)/u+2;break;case r:l=(i-s)/u+4;break}l/=6}return t.h=l,t.s=c,t.l=h,t}getRGB(t,e=se.workingColorSpace){return se.fromWorkingColorSpace(Je.copy(this),e),t.r=Je.r,t.g=Je.g,t.b=Je.b,t}getStyle(t=Ke){se.fromWorkingColorSpace(Je.copy(this),t);let e=Je.r,i=Je.g,s=Je.b;return t!==Ke?`color(${t} ${e.toFixed(3)} ${i.toFixed(3)} ${s.toFixed(3)})`:`rgb(${Math.round(e*255)},${Math.round(i*255)},${Math.round(s*255)})`}offsetHSL(t,e,i){return this.getHSL(Xi),this.setHSL(Xi.h+t,Xi.s+e,Xi.l+i)}add(t){return this.r+=t.r,this.g+=t.g,this.b+=t.b,this}addColors(t,e){return this.r=t.r+e.r,this.g=t.g+e.g,this.b=t.b+e.b,this}addScalar(t){return this.r+=t,this.g+=t,this.b+=t,this}sub(t){return this.r=Math.max(0,this.r-t.r),this.g=Math.max(0,this.g-t.g),this.b=Math.max(0,this.b-t.b),this}multiply(t){return this.r*=t.r,this.g*=t.g,this.b*=t.b,this}multiplyScalar(t){return this.r*=t,this.g*=t,this.b*=t,this}lerp(t,e){return this.r+=(t.r-this.r)*e,this.g+=(t.g-this.g)*e,this.b+=(t.b-this.b)*e,this}lerpColors(t,e,i){return this.r=t.r+(e.r-t.r)*i,this.g=t.g+(e.g-t.g)*i,this.b=t.b+(e.b-t.b)*i,this}lerpHSL(t,e){this.getHSL(Xi),t.getHSL(so);let i=Vl(Xi.h,so.h,e),s=Vl(Xi.s,so.s,e),r=Vl(Xi.l,so.l,e);return this.setHSL(i,s,r),this}setFromVector3(t){return this.r=t.x,this.g=t.y,this.b=t.z,this}applyMatrix3(t){let e=this.r,i=this.g,s=this.b,r=t.elements;return this.r=r[0]*e+r[3]*i+r[6]*s,this.g=r[1]*e+r[4]*i+r[7]*s,this.b=r[2]*e+r[5]*i+r[8]*s,this}equals(t){return t.r===this.r&&t.g===this.g&&t.b===this.b}fromArray(t,e=0){return this.r=t[e],this.g=t[e+1],this.b=t[e+2],this}toArray(t=[],e=0){return t[e]=this.r,t[e+1]=this.g,t[e+2]=this.b,t}fromBufferAttribute(t,e){return this.r=t.getX(e),this.g=t.getY(e),this.b=t.getZ(e),this}toJSON(){return this.getHex()}*[Symbol.iterator](){yield this.r,yield this.g,yield this.b}},Je=new Pt;Pt.NAMES=bf;var ym=0,es=class extends ji{static get type(){return"Material"}get type(){return this.constructor.type}set type(t){}constructor(){super(),this.isMaterial=!0,Object.defineProperty(this,"id",{value:ym++}),this.uuid=Ei(),this.name="",this.blending=br,this.side=Qi,this.vertexColors=!1,this.opacity=1,this.transparent=!1,this.alphaHash=!1,this.blendSrc=ua,this.blendDst=bc,this.blendEquation=Si,this.blendSrcAlpha=null,this.blendDstAlpha=null,this.blendEquationAlpha=null,this.blendColor=new Pt(0,0,0),this.blendAlpha=0,this.depthFunc=Er,this.depthTest=!0,this.depthWrite=!0,this.stencilWriteMask=255,this.stencilFunc=td,this.stencilRef=0,this.stencilFuncMask=255,this.stencilFail=tr,this.stencilZFail=tr,this.stencilZPass=tr,this.stencilWrite=!1,this.clippingPlanes=null,this.clipIntersection=!1,this.clipShadows=!1,this.shadowSide=null,this.colorWrite=!0,this.precision=null,this.polygonOffset=!1,this.polygonOffsetFactor=0,this.polygonOffsetUnits=0,this.dithering=!1,this.alphaToCoverage=!1,this.premultipliedAlpha=!1,this.forceSinglePass=!1,this.visible=!0,this.toneMapped=!0,this.userData={},this.version=0,this._alphaTest=0}get alphaTest(){return this._alphaTest}set alphaTest(t){this._alphaTest>0!=t>0&&this.version++,this._alphaTest=t}onBeforeRender(){}onBeforeCompile(){}customProgramCacheKey(){return this.onBeforeCompile.toString()}setValues(t){if(t!==void 0)for(let e in t){let i=t[e];if(i===void 0){console.warn(`THREE.Material: parameter '${e}' has value of undefined.`);continue}let s=this[e];if(s===void 0){console.warn(`THREE.Material: '${e}' is not a property of THREE.${this.type}.`);continue}s&&s.isColor?s.set(i):s&&s.isVector3&&i&&i.isVector3?s.copy(i):this[e]=i}}toJSON(t){let e=t===void 0||typeof t=="string";e&&(t={textures:{},images:{}});let i={metadata:{version:4.6,type:"Material",generator:"Material.toJSON"}};i.uuid=this.uuid,i.type=this.type,this.name!==""&&(i.name=this.name),this.color&&this.color.isColor&&(i.color=this.color.getHex()),this.roughness!==void 0&&(i.roughness=this.roughness),this.metalness!==void 0&&(i.metalness=this.metalness),this.sheen!==void 0&&(i.sheen=this.sheen),this.sheenColor&&this.sheenColor.isColor&&(i.sheenColor=this.sheenColor.getHex()),this.sheenRoughness!==void 0&&(i.sheenRoughness=this.sheenRoughness),this.emissive&&this.emissive.isColor&&(i.emissive=this.emissive.getHex()),this.emissiveIntensity!==void 0&&this.emissiveIntensity!==1&&(i.emissiveIntensity=this.emissiveIntensity),this.specular&&this.specular.isColor&&(i.specular=this.specular.getHex()),this.specularIntensity!==void 0&&(i.specularIntensity=this.specularIntensity),this.specularColor&&this.specularColor.isColor&&(i.specularColor=this.specularColor.getHex()),this.shininess!==void 0&&(i.shininess=this.shininess),this.clearcoat!==void 0&&(i.clearcoat=this.clearcoat),this.clearcoatRoughness!==void 0&&(i.clearcoatRoughness=this.clearcoatRoughness),this.clearcoatMap&&this.clearcoatMap.isTexture&&(i.clearcoatMap=this.clearcoatMap.toJSON(t).uuid),this.clearcoatRoughnessMap&&this.clearcoatRoughnessMap.isTexture&&(i.clearcoatRoughnessMap=this.clearcoatRoughnessMap.toJSON(t).uuid),this.clearcoatNormalMap&&this.clearcoatNormalMap.isTexture&&(i.clearcoatNormalMap=this.clearcoatNormalMap.toJSON(t).uuid,i.clearcoatNormalScale=this.clearcoatNormalScale.toArray()),this.dispersion!==void 0&&(i.dispersion=this.dispersion),this.iridescence!==void 0&&(i.iridescence=this.iridescence),this.iridescenceIOR!==void 0&&(i.iridescenceIOR=this.iridescenceIOR),this.iridescenceThicknessRange!==void 0&&(i.iridescenceThicknessRange=this.iridescenceThicknessRange),this.iridescenceMap&&this.iridescenceMap.isTexture&&(i.iridescenceMap=this.iridescenceMap.toJSON(t).uuid),this.iridescenceThicknessMap&&this.iridescenceThicknessMap.isTexture&&(i.iridescenceThicknessMap=this.iridescenceThicknessMap.toJSON(t).uuid),this.anisotropy!==void 0&&(i.anisotropy=this.anisotropy),this.anisotropyRotation!==void 0&&(i.anisotropyRotation=this.anisotropyRotation),this.anisotropyMap&&this.anisotropyMap.isTexture&&(i.anisotropyMap=this.anisotropyMap.toJSON(t).uuid),this.map&&this.map.isTexture&&(i.map=this.map.toJSON(t).uuid),this.matcap&&this.matcap.isTexture&&(i.matcap=this.matcap.toJSON(t).uuid),this.alphaMap&&this.alphaMap.isTexture&&(i.alphaMap=this.alphaMap.toJSON(t).uuid),this.lightMap&&this.lightMap.isTexture&&(i.lightMap=this.lightMap.toJSON(t).uuid,i.lightMapIntensity=this.lightMapIntensity),this.aoMap&&this.aoMap.isTexture&&(i.aoMap=this.aoMap.toJSON(t).uuid,i.aoMapIntensity=this.aoMapIntensity),this.bumpMap&&this.bumpMap.isTexture&&(i.bumpMap=this.bumpMap.toJSON(t).uuid,i.bumpScale=this.bumpScale),this.normalMap&&this.normalMap.isTexture&&(i.normalMap=this.normalMap.toJSON(t).uuid,i.normalMapType=this.normalMapType,i.normalScale=this.normalScale.toArray()),this.displacementMap&&this.displacementMap.isTexture&&(i.displacementMap=this.displacementMap.toJSON(t).uuid,i.displacementScale=this.displacementScale,i.displacementBias=this.displacementBias),this.roughnessMap&&this.roughnessMap.isTexture&&(i.roughnessMap=this.roughnessMap.toJSON(t).uuid),this.metalnessMap&&this.metalnessMap.isTexture&&(i.metalnessMap=this.metalnessMap.toJSON(t).uuid),this.emissiveMap&&this.emissiveMap.isTexture&&(i.emissiveMap=this.emissiveMap.toJSON(t).uuid),this.specularMap&&this.specularMap.isTexture&&(i.specularMap=this.specularMap.toJSON(t).uuid),this.specularIntensityMap&&this.specularIntensityMap.isTexture&&(i.specularIntensityMap=this.specularIntensityMap.toJSON(t).uuid),this.specularColorMap&&this.specularColorMap.isTexture&&(i.specularColorMap=this.specularColorMap.toJSON(t).uuid),this.envMap&&this.envMap.isTexture&&(i.envMap=this.envMap.toJSON(t).uuid,this.combine!==void 0&&(i.combine=this.combine)),this.envMapRotation!==void 0&&(i.envMapRotation=this.envMapRotation.toArray()),this.envMapIntensity!==void 0&&(i.envMapIntensity=this.envMapIntensity),this.reflectivity!==void 0&&(i.reflectivity=this.reflectivity),this.refractionRatio!==void 0&&(i.refractionRatio=this.refractionRatio),this.gradientMap&&this.gradientMap.isTexture&&(i.gradientMap=this.gradientMap.toJSON(t).uuid),this.transmission!==void 0&&(i.transmission=this.transmission),this.transmissionMap&&this.transmissionMap.isTexture&&(i.transmissionMap=this.transmissionMap.toJSON(t).uuid),this.thickness!==void 0&&(i.thickness=this.thickness),this.thicknessMap&&this.thicknessMap.isTexture&&(i.thicknessMap=this.thicknessMap.toJSON(t).uuid),this.attenuationDistance!==void 0&&this.attenuationDistance!==1/0&&(i.attenuationDistance=this.attenuationDistance),this.attenuationColor!==void 0&&(i.attenuationColor=this.attenuationColor.getHex()),this.size!==void 0&&(i.size=this.size),this.shadowSide!==null&&(i.shadowSide=this.shadowSide),this.sizeAttenuation!==void 0&&(i.sizeAttenuation=this.sizeAttenuation),this.blending!==br&&(i.blending=this.blending),this.side!==Qi&&(i.side=this.side),this.vertexColors===!0&&(i.vertexColors=!0),this.opacity<1&&(i.opacity=this.opacity),this.transparent===!0&&(i.transparent=!0),this.blendSrc!==ua&&(i.blendSrc=this.blendSrc),this.blendDst!==bc&&(i.blendDst=this.blendDst),this.blendEquation!==Si&&(i.blendEquation=this.blendEquation),this.blendSrcAlpha!==null&&(i.blendSrcAlpha=this.blendSrcAlpha),this.blendDstAlpha!==null&&(i.blendDstAlpha=this.blendDstAlpha),this.blendEquationAlpha!==null&&(i.blendEquationAlpha=this.blendEquationAlpha),this.blendColor&&this.blendColor.isColor&&(i.blendColor=this.blendColor.getHex()),this.blendAlpha!==0&&(i.blendAlpha=this.blendAlpha),this.depthFunc!==Er&&(i.depthFunc=this.depthFunc),this.depthTest===!1&&(i.depthTest=this.depthTest),this.depthWrite===!1&&(i.depthWrite=this.depthWrite),this.colorWrite===!1&&(i.colorWrite=this.colorWrite),this.stencilWriteMask!==255&&(i.stencilWriteMask=this.stencilWriteMask),this.stencilFunc!==td&&(i.stencilFunc=this.stencilFunc),this.stencilRef!==0&&(i.stencilRef=this.stencilRef),this.stencilFuncMask!==255&&(i.stencilFuncMask=this.stencilFuncMask),this.stencilFail!==tr&&(i.stencilFail=this.stencilFail),this.stencilZFail!==tr&&(i.stencilZFail=this.stencilZFail),this.stencilZPass!==tr&&(i.stencilZPass=this.stencilZPass),this.stencilWrite===!0&&(i.stencilWrite=this.stencilWrite),this.rotation!==void 0&&this.rotation!==0&&(i.rotation=this.rotation),this.polygonOffset===!0&&(i.polygonOffset=!0),this.polygonOffsetFactor!==0&&(i.polygonOffsetFactor=this.polygonOffsetFactor),this.polygonOffsetUnits!==0&&(i.polygonOffsetUnits=this.polygonOffsetUnits),this.linewidth!==void 0&&this.linewidth!==1&&(i.linewidth=this.linewidth),this.dashSize!==void 0&&(i.dashSize=this.dashSize),this.gapSize!==void 0&&(i.gapSize=this.gapSize),this.scale!==void 0&&(i.scale=this.scale),this.dithering===!0&&(i.dithering=!0),this.alphaTest>0&&(i.alphaTest=this.alphaTest),this.alphaHash===!0&&(i.alphaHash=!0),this.alphaToCoverage===!0&&(i.alphaToCoverage=!0),this.premultipliedAlpha===!0&&(i.premultipliedAlpha=!0),this.forceSinglePass===!0&&(i.forceSinglePass=!0),this.wireframe===!0&&(i.wireframe=!0),this.wireframeLinewidth>1&&(i.wireframeLinewidth=this.wireframeLinewidth),this.wireframeLinecap!=="round"&&(i.wireframeLinecap=this.wireframeLinecap),this.wireframeLinejoin!=="round"&&(i.wireframeLinejoin=this.wireframeLinejoin),this.flatShading===!0&&(i.flatShading=!0),this.visible===!1&&(i.visible=!1),this.toneMapped===!1&&(i.toneMapped=!1),this.fog===!1&&(i.fog=!1),Object.keys(this.userData).length>0&&(i.userData=this.userData);function s(r){let a=[];for(let o in r){let l=r[o];delete l.metadata,a.push(l)}return a}if(e){let r=s(t.textures),a=s(t.images);r.length>0&&(i.textures=r),a.length>0&&(i.images=a)}return i}clone(){return new this.constructor().copy(this)}copy(t){this.name=t.name,this.blending=t.blending,this.side=t.side,this.vertexColors=t.vertexColors,this.opacity=t.opacity,this.transparent=t.transparent,this.blendSrc=t.blendSrc,this.blendDst=t.blendDst,this.blendEquation=t.blendEquation,this.blendSrcAlpha=t.blendSrcAlpha,this.blendDstAlpha=t.blendDstAlpha,this.blendEquationAlpha=t.blendEquationAlpha,this.blendColor.copy(t.blendColor),this.blendAlpha=t.blendAlpha,this.depthFunc=t.depthFunc,this.depthTest=t.depthTest,this.depthWrite=t.depthWrite,this.stencilWriteMask=t.stencilWriteMask,this.stencilFunc=t.stencilFunc,this.stencilRef=t.stencilRef,this.stencilFuncMask=t.stencilFuncMask,this.stencilFail=t.stencilFail,this.stencilZFail=t.stencilZFail,this.stencilZPass=t.stencilZPass,this.stencilWrite=t.stencilWrite;let e=t.clippingPlanes,i=null;if(e!==null){let s=e.length;i=new Array(s);for(let r=0;r!==s;++r)i[r]=e[r].clone()}return this.clippingPlanes=i,this.clipIntersection=t.clipIntersection,this.clipShadows=t.clipShadows,this.shadowSide=t.shadowSide,this.colorWrite=t.colorWrite,this.precision=t.precision,this.polygonOffset=t.polygonOffset,this.polygonOffsetFactor=t.polygonOffsetFactor,this.polygonOffsetUnits=t.polygonOffsetUnits,this.dithering=t.dithering,this.alphaTest=t.alphaTest,this.alphaHash=t.alphaHash,this.alphaToCoverage=t.alphaToCoverage,this.premultipliedAlpha=t.premultipliedAlpha,this.forceSinglePass=t.forceSinglePass,this.visible=t.visible,this.toneMapped=t.toneMapped,this.userData=JSON.parse(JSON.stringify(t.userData)),this}dispose(){this.dispatchEvent({type:"dispose"})}set needsUpdate(t){t===!0&&this.version++}onBuild(){console.warn("Material: onBuild() has been removed.")}},Ve=class extends es{static get type(){return"MeshBasicMaterial"}constructor(t){super(),this.isMeshBasicMaterial=!0,this.color=new Pt(16777215),this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.specularMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new ti,this.combine=of,this.reflectivity=1,this.refractionRatio=.98,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.lightMap=t.lightMap,this.lightMapIntensity=t.lightMapIntensity,this.aoMap=t.aoMap,this.aoMapIntensity=t.aoMapIntensity,this.specularMap=t.specularMap,this.alphaMap=t.alphaMap,this.envMap=t.envMap,this.envMapRotation.copy(t.envMapRotation),this.combine=t.combine,this.reflectivity=t.reflectivity,this.refractionRatio=t.refractionRatio,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.wireframeLinecap=t.wireframeLinecap,this.wireframeLinejoin=t.wireframeLinejoin,this.fog=t.fog,this}};var Le=new I,ro=new j,an=class{constructor(t,e,i=!1){if(Array.isArray(t))throw new TypeError("THREE.BufferAttribute: array should be a Typed Array.");this.isBufferAttribute=!0,this.name="",this.array=t,this.itemSize=e,this.count=t!==void 0?t.length/e:0,this.normalized=i,this.usage=ah,this.updateRanges=[],this.gpuType=Kn,this.version=0}onUploadCallback(){}set needsUpdate(t){t===!0&&this.version++}setUsage(t){return this.usage=t,this}addUpdateRange(t,e){this.updateRanges.push({start:t,count:e})}clearUpdateRanges(){this.updateRanges.length=0}copy(t){return this.name=t.name,this.array=new t.array.constructor(t.array),this.itemSize=t.itemSize,this.count=t.count,this.normalized=t.normalized,this.usage=t.usage,this.gpuType=t.gpuType,this}copyAt(t,e,i){t*=this.itemSize,i*=e.itemSize;for(let s=0,r=this.itemSize;s<r;s++)this.array[t+s]=e.array[i+s];return this}copyArray(t){return this.array.set(t),this}applyMatrix3(t){if(this.itemSize===2)for(let e=0,i=this.count;e<i;e++)ro.fromBufferAttribute(this,e),ro.applyMatrix3(t),this.setXY(e,ro.x,ro.y);else if(this.itemSize===3)for(let e=0,i=this.count;e<i;e++)Le.fromBufferAttribute(this,e),Le.applyMatrix3(t),this.setXYZ(e,Le.x,Le.y,Le.z);return this}applyMatrix4(t){for(let e=0,i=this.count;e<i;e++)Le.fromBufferAttribute(this,e),Le.applyMatrix4(t),this.setXYZ(e,Le.x,Le.y,Le.z);return this}applyNormalMatrix(t){for(let e=0,i=this.count;e<i;e++)Le.fromBufferAttribute(this,e),Le.applyNormalMatrix(t),this.setXYZ(e,Le.x,Le.y,Le.z);return this}transformDirection(t){for(let e=0,i=this.count;e<i;e++)Le.fromBufferAttribute(this,e),Le.transformDirection(t),this.setXYZ(e,Le.x,Le.y,Le.z);return this}set(t,e=0){return this.array.set(t,e),this}getComponent(t,e){let i=this.array[t*this.itemSize+e];return this.normalized&&(i=Zn(i,this.array)),i}setComponent(t,e,i){return this.normalized&&(i=ge(i,this.array)),this.array[t*this.itemSize+e]=i,this}getX(t){let e=this.array[t*this.itemSize];return this.normalized&&(e=Zn(e,this.array)),e}setX(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize]=e,this}getY(t){let e=this.array[t*this.itemSize+1];return this.normalized&&(e=Zn(e,this.array)),e}setY(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize+1]=e,this}getZ(t){let e=this.array[t*this.itemSize+2];return this.normalized&&(e=Zn(e,this.array)),e}setZ(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize+2]=e,this}getW(t){let e=this.array[t*this.itemSize+3];return this.normalized&&(e=Zn(e,this.array)),e}setW(t,e){return this.normalized&&(e=ge(e,this.array)),this.array[t*this.itemSize+3]=e,this}setXY(t,e,i){return t*=this.itemSize,this.normalized&&(e=ge(e,this.array),i=ge(i,this.array)),this.array[t+0]=e,this.array[t+1]=i,this}setXYZ(t,e,i,s){return t*=this.itemSize,this.normalized&&(e=ge(e,this.array),i=ge(i,this.array),s=ge(s,this.array)),this.array[t+0]=e,this.array[t+1]=i,this.array[t+2]=s,this}setXYZW(t,e,i,s,r){return t*=this.itemSize,this.normalized&&(e=ge(e,this.array),i=ge(i,this.array),s=ge(s,this.array),r=ge(r,this.array)),this.array[t+0]=e,this.array[t+1]=i,this.array[t+2]=s,this.array[t+3]=r,this}onUpload(t){return this.onUploadCallback=t,this}clone(){return new this.constructor(this.array,this.itemSize).copy(this)}toJSON(){let t={itemSize:this.itemSize,type:this.array.constructor.name,array:Array.from(this.array),normalized:this.normalized};return this.name!==""&&(t.name=this.name),this.usage!==ah&&(t.usage=this.usage),t}};var Uo=class extends an{constructor(t,e,i){super(new Uint16Array(t),e,i)}};var No=class extends an{constructor(t,e,i){super(new Uint32Array(t),e,i)}};var _e=class extends an{constructor(t,e,i){super(new Float32Array(t),e,i)}},_m=0,Cn=new pe,ac=new ln,hr=new I,xn=new jn,jr=new jn,ze=new I,tn=class n extends ji{constructor(){super(),this.isBufferGeometry=!0,Object.defineProperty(this,"id",{value:_m++}),this.uuid=Ei(),this.name="",this.type="BufferGeometry",this.index=null,this.indirect=null,this.attributes={},this.morphAttributes={},this.morphTargetsRelative=!1,this.groups=[],this.boundingBox=null,this.boundingSphere=null,this.drawRange={start:0,count:1/0},this.userData={}}getIndex(){return this.index}setIndex(t){return Array.isArray(t)?this.index=new(_f(t)?No:Uo)(t,1):this.index=t,this}setIndirect(t){return this.indirect=t,this}getIndirect(){return this.indirect}getAttribute(t){return this.attributes[t]}setAttribute(t,e){return this.attributes[t]=e,this}deleteAttribute(t){return delete this.attributes[t],this}hasAttribute(t){return this.attributes[t]!==void 0}addGroup(t,e,i=0){this.groups.push({start:t,count:e,materialIndex:i})}clearGroups(){this.groups=[]}setDrawRange(t,e){this.drawRange.start=t,this.drawRange.count=e}applyMatrix4(t){let e=this.attributes.position;e!==void 0&&(e.applyMatrix4(t),e.needsUpdate=!0);let i=this.attributes.normal;if(i!==void 0){let r=new Kt().getNormalMatrix(t);i.applyNormalMatrix(r),i.needsUpdate=!0}let s=this.attributes.tangent;return s!==void 0&&(s.transformDirection(t),s.needsUpdate=!0),this.boundingBox!==null&&this.computeBoundingBox(),this.boundingSphere!==null&&this.computeBoundingSphere(),this}applyQuaternion(t){return Cn.makeRotationFromQuaternion(t),this.applyMatrix4(Cn),this}rotateX(t){return Cn.makeRotationX(t),this.applyMatrix4(Cn),this}rotateY(t){return Cn.makeRotationY(t),this.applyMatrix4(Cn),this}rotateZ(t){return Cn.makeRotationZ(t),this.applyMatrix4(Cn),this}translate(t,e,i){return Cn.makeTranslation(t,e,i),this.applyMatrix4(Cn),this}scale(t,e,i){return Cn.makeScale(t,e,i),this.applyMatrix4(Cn),this}lookAt(t){return ac.lookAt(t),ac.updateMatrix(),this.applyMatrix4(ac.matrix),this}center(){return this.computeBoundingBox(),this.boundingBox.getCenter(hr).negate(),this.translate(hr.x,hr.y,hr.z),this}setFromPoints(t){let e=this.getAttribute("position");if(e===void 0){let i=[];for(let s=0,r=t.length;s<r;s++){let a=t[s];i.push(a.x,a.y,a.z||0)}this.setAttribute("position",new _e(i,3))}else{for(let i=0,s=e.count;i<s;i++){let r=t[i];e.setXYZ(i,r.x,r.y,r.z||0)}t.length>e.count&&console.warn("THREE.BufferGeometry: Buffer size too small for points data. Use .dispose() and create a new geometry."),e.needsUpdate=!0}return this}computeBoundingBox(){this.boundingBox===null&&(this.boundingBox=new jn);let t=this.attributes.position,e=this.morphAttributes.position;if(t&&t.isGLBufferAttribute){console.error("THREE.BufferGeometry.computeBoundingBox(): GLBufferAttribute requires a manual bounding box.",this),this.boundingBox.set(new I(-1/0,-1/0,-1/0),new I(1/0,1/0,1/0));return}if(t!==void 0){if(this.boundingBox.setFromBufferAttribute(t),e)for(let i=0,s=e.length;i<s;i++){let r=e[i];xn.setFromBufferAttribute(r),this.morphTargetsRelative?(ze.addVectors(this.boundingBox.min,xn.min),this.boundingBox.expandByPoint(ze),ze.addVectors(this.boundingBox.max,xn.max),this.boundingBox.expandByPoint(ze)):(this.boundingBox.expandByPoint(xn.min),this.boundingBox.expandByPoint(xn.max))}}else this.boundingBox.makeEmpty();(isNaN(this.boundingBox.min.x)||isNaN(this.boundingBox.min.y)||isNaN(this.boundingBox.min.z))&&console.error('THREE.BufferGeometry.computeBoundingBox(): Computed min/max have NaN values. The "position" attribute is likely to have NaN values.',this)}computeBoundingSphere(){this.boundingSphere===null&&(this.boundingSphere=new Ns);let t=this.attributes.position,e=this.morphAttributes.position;if(t&&t.isGLBufferAttribute){console.error("THREE.BufferGeometry.computeBoundingSphere(): GLBufferAttribute requires a manual bounding sphere.",this),this.boundingSphere.set(new I,1/0);return}if(t){let i=this.boundingSphere.center;if(xn.setFromBufferAttribute(t),e)for(let r=0,a=e.length;r<a;r++){let o=e[r];jr.setFromBufferAttribute(o),this.morphTargetsRelative?(ze.addVectors(xn.min,jr.min),xn.expandByPoint(ze),ze.addVectors(xn.max,jr.max),xn.expandByPoint(ze)):(xn.expandByPoint(jr.min),xn.expandByPoint(jr.max))}xn.getCenter(i);let s=0;for(let r=0,a=t.count;r<a;r++)ze.fromBufferAttribute(t,r),s=Math.max(s,i.distanceToSquared(ze));if(e)for(let r=0,a=e.length;r<a;r++){let o=e[r],l=this.morphTargetsRelative;for(let c=0,h=o.count;c<h;c++)ze.fromBufferAttribute(o,c),l&&(hr.fromBufferAttribute(t,c),ze.add(hr)),s=Math.max(s,i.distanceToSquared(ze))}this.boundingSphere.radius=Math.sqrt(s),isNaN(this.boundingSphere.radius)&&console.error('THREE.BufferGeometry.computeBoundingSphere(): Computed radius is NaN. The "position" attribute is likely to have NaN values.',this)}}computeTangents(){let t=this.index,e=this.attributes;if(t===null||e.position===void 0||e.normal===void 0||e.uv===void 0){console.error("THREE.BufferGeometry: .computeTangents() failed. Missing required attributes (index, position, normal or uv)");return}let i=e.position,s=e.normal,r=e.uv;this.hasAttribute("tangent")===!1&&this.setAttribute("tangent",new an(new Float32Array(4*i.count),4));let a=this.getAttribute("tangent"),o=[],l=[];for(let P=0;P<i.count;P++)o[P]=new I,l[P]=new I;let c=new I,h=new I,u=new I,d=new j,f=new j,g=new j,x=new I,m=new I;function p(P,w,_){c.fromBufferAttribute(i,P),h.fromBufferAttribute(i,w),u.fromBufferAttribute(i,_),d.fromBufferAttribute(r,P),f.fromBufferAttribute(r,w),g.fromBufferAttribute(r,_),h.sub(c),u.sub(c),f.sub(d),g.sub(d);let R=1/(f.x*g.y-g.x*f.y);isFinite(R)&&(x.copy(h).multiplyScalar(g.y).addScaledVector(u,-f.y).multiplyScalar(R),m.copy(u).multiplyScalar(f.x).addScaledVector(h,-g.x).multiplyScalar(R),o[P].add(x),o[w].add(x),o[_].add(x),l[P].add(m),l[w].add(m),l[_].add(m))}let T=this.groups;T.length===0&&(T=[{start:0,count:t.count}]);for(let P=0,w=T.length;P<w;++P){let _=T[P],R=_.start,G=_.count;for(let N=R,z=R+G;N<z;N+=3)p(t.getX(N+0),t.getX(N+1),t.getX(N+2))}let M=new I,y=new I,D=new I,S=new I;function C(P){D.fromBufferAttribute(s,P),S.copy(D);let w=o[P];M.copy(w),M.sub(D.multiplyScalar(D.dot(w))).normalize(),y.crossVectors(S,w);let R=y.dot(l[P])<0?-1:1;a.setXYZW(P,M.x,M.y,M.z,R)}for(let P=0,w=T.length;P<w;++P){let _=T[P],R=_.start,G=_.count;for(let N=R,z=R+G;N<z;N+=3)C(t.getX(N+0)),C(t.getX(N+1)),C(t.getX(N+2))}}computeVertexNormals(){let t=this.index,e=this.getAttribute("position");if(e!==void 0){let i=this.getAttribute("normal");if(i===void 0)i=new an(new Float32Array(e.count*3),3),this.setAttribute("normal",i);else for(let d=0,f=i.count;d<f;d++)i.setXYZ(d,0,0,0);let s=new I,r=new I,a=new I,o=new I,l=new I,c=new I,h=new I,u=new I;if(t)for(let d=0,f=t.count;d<f;d+=3){let g=t.getX(d+0),x=t.getX(d+1),m=t.getX(d+2);s.fromBufferAttribute(e,g),r.fromBufferAttribute(e,x),a.fromBufferAttribute(e,m),h.subVectors(a,r),u.subVectors(s,r),h.cross(u),o.fromBufferAttribute(i,g),l.fromBufferAttribute(i,x),c.fromBufferAttribute(i,m),o.add(h),l.add(h),c.add(h),i.setXYZ(g,o.x,o.y,o.z),i.setXYZ(x,l.x,l.y,l.z),i.setXYZ(m,c.x,c.y,c.z)}else for(let d=0,f=e.count;d<f;d+=3)s.fromBufferAttribute(e,d+0),r.fromBufferAttribute(e,d+1),a.fromBufferAttribute(e,d+2),h.subVectors(a,r),u.subVectors(s,r),h.cross(u),i.setXYZ(d+0,h.x,h.y,h.z),i.setXYZ(d+1,h.x,h.y,h.z),i.setXYZ(d+2,h.x,h.y,h.z);this.normalizeNormals(),i.needsUpdate=!0}}normalizeNormals(){let t=this.attributes.normal;for(let e=0,i=t.count;e<i;e++)ze.fromBufferAttribute(t,e),ze.normalize(),t.setXYZ(e,ze.x,ze.y,ze.z)}toNonIndexed(){function t(o,l){let c=o.array,h=o.itemSize,u=o.normalized,d=new c.constructor(l.length*h),f=0,g=0;for(let x=0,m=l.length;x<m;x++){o.isInterleavedBufferAttribute?f=l[x]*o.data.stride+o.offset:f=l[x]*h;for(let p=0;p<h;p++)d[g++]=c[f++]}return new an(d,h,u)}if(this.index===null)return console.warn("THREE.BufferGeometry.toNonIndexed(): BufferGeometry is already non-indexed."),this;let e=new n,i=this.index.array,s=this.attributes;for(let o in s){let l=s[o],c=t(l,i);e.setAttribute(o,c)}let r=this.morphAttributes;for(let o in r){let l=[],c=r[o];for(let h=0,u=c.length;h<u;h++){let d=c[h],f=t(d,i);l.push(f)}e.morphAttributes[o]=l}e.morphTargetsRelative=this.morphTargetsRelative;let a=this.groups;for(let o=0,l=a.length;o<l;o++){let c=a[o];e.addGroup(c.start,c.count,c.materialIndex)}return e}toJSON(){let t={metadata:{version:4.6,type:"BufferGeometry",generator:"BufferGeometry.toJSON"}};if(t.uuid=this.uuid,t.type=this.type,this.name!==""&&(t.name=this.name),Object.keys(this.userData).length>0&&(t.userData=this.userData),this.parameters!==void 0){let l=this.parameters;for(let c in l)l[c]!==void 0&&(t[c]=l[c]);return t}t.data={attributes:{}};let e=this.index;e!==null&&(t.data.index={type:e.array.constructor.name,array:Array.prototype.slice.call(e.array)});let i=this.attributes;for(let l in i){let c=i[l];t.data.attributes[l]=c.toJSON(t.data)}let s={},r=!1;for(let l in this.morphAttributes){let c=this.morphAttributes[l],h=[];for(let u=0,d=c.length;u<d;u++){let f=c[u];h.push(f.toJSON(t.data))}h.length>0&&(s[l]=h,r=!0)}r&&(t.data.morphAttributes=s,t.data.morphTargetsRelative=this.morphTargetsRelative);let a=this.groups;a.length>0&&(t.data.groups=JSON.parse(JSON.stringify(a)));let o=this.boundingSphere;return o!==null&&(t.data.boundingSphere={center:o.center.toArray(),radius:o.radius}),t}clone(){return new this.constructor().copy(this)}copy(t){this.index=null,this.attributes={},this.morphAttributes={},this.groups=[],this.boundingBox=null,this.boundingSphere=null;let e={};this.name=t.name;let i=t.index;i!==null&&this.setIndex(i.clone(e));let s=t.attributes;for(let c in s){let h=s[c];this.setAttribute(c,h.clone(e))}let r=t.morphAttributes;for(let c in r){let h=[],u=r[c];for(let d=0,f=u.length;d<f;d++)h.push(u[d].clone(e));this.morphAttributes[c]=h}this.morphTargetsRelative=t.morphTargetsRelative;let a=t.groups;for(let c=0,h=a.length;c<h;c++){let u=a[c];this.addGroup(u.start,u.count,u.materialIndex)}let o=t.boundingBox;o!==null&&(this.boundingBox=o.clone());let l=t.boundingSphere;return l!==null&&(this.boundingSphere=l.clone()),this.drawRange.start=t.drawRange.start,this.drawRange.count=t.drawRange.count,this.userData=t.userData,this}dispose(){this.dispatchEvent({type:"dispose"})}},xd=new pe,Cs=new uh,ao=new Ns,vd=new I,oo=new I,lo=new I,co=new I,oc=new I,ho=new I,yd=new I,uo=new I,Ot=class extends ln{constructor(t=new tn,e=new Ve){super(),this.isMesh=!0,this.type="Mesh",this.geometry=t,this.material=e,this.updateMorphTargets()}copy(t,e){return super.copy(t,e),t.morphTargetInfluences!==void 0&&(this.morphTargetInfluences=t.morphTargetInfluences.slice()),t.morphTargetDictionary!==void 0&&(this.morphTargetDictionary=Object.assign({},t.morphTargetDictionary)),this.material=Array.isArray(t.material)?t.material.slice():t.material,this.geometry=t.geometry,this}updateMorphTargets(){let e=this.geometry.morphAttributes,i=Object.keys(e);if(i.length>0){let s=e[i[0]];if(s!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let r=0,a=s.length;r<a;r++){let o=s[r].name||String(r);this.morphTargetInfluences.push(0),this.morphTargetDictionary[o]=r}}}}getVertexPosition(t,e){let i=this.geometry,s=i.attributes.position,r=i.morphAttributes.position,a=i.morphTargetsRelative;e.fromBufferAttribute(s,t);let o=this.morphTargetInfluences;if(r&&o){ho.set(0,0,0);for(let l=0,c=r.length;l<c;l++){let h=o[l],u=r[l];h!==0&&(oc.fromBufferAttribute(u,t),a?ho.addScaledVector(oc,h):ho.addScaledVector(oc.sub(e),h))}e.add(ho)}return e}raycast(t,e){let i=this.geometry,s=this.material,r=this.matrixWorld;s!==void 0&&(i.boundingSphere===null&&i.computeBoundingSphere(),ao.copy(i.boundingSphere),ao.applyMatrix4(r),Cs.copy(t.ray).recast(t.near),!(ao.containsPoint(Cs.origin)===!1&&(Cs.intersectSphere(ao,vd)===null||Cs.origin.distanceToSquared(vd)>(t.far-t.near)**2))&&(xd.copy(r).invert(),Cs.copy(t.ray).applyMatrix4(xd),!(i.boundingBox!==null&&Cs.intersectsBox(i.boundingBox)===!1)&&this._computeIntersections(t,e,Cs)))}_computeIntersections(t,e,i){let s,r=this.geometry,a=this.material,o=r.index,l=r.attributes.position,c=r.attributes.uv,h=r.attributes.uv1,u=r.attributes.normal,d=r.groups,f=r.drawRange;if(o!==null)if(Array.isArray(a))for(let g=0,x=d.length;g<x;g++){let m=d[g],p=a[m.materialIndex],T=Math.max(m.start,f.start),M=Math.min(o.count,Math.min(m.start+m.count,f.start+f.count));for(let y=T,D=M;y<D;y+=3){let S=o.getX(y),C=o.getX(y+1),P=o.getX(y+2);s=fo(this,p,t,i,c,h,u,S,C,P),s&&(s.faceIndex=Math.floor(y/3),s.face.materialIndex=m.materialIndex,e.push(s))}}else{let g=Math.max(0,f.start),x=Math.min(o.count,f.start+f.count);for(let m=g,p=x;m<p;m+=3){let T=o.getX(m),M=o.getX(m+1),y=o.getX(m+2);s=fo(this,a,t,i,c,h,u,T,M,y),s&&(s.faceIndex=Math.floor(m/3),e.push(s))}}else if(l!==void 0)if(Array.isArray(a))for(let g=0,x=d.length;g<x;g++){let m=d[g],p=a[m.materialIndex],T=Math.max(m.start,f.start),M=Math.min(l.count,Math.min(m.start+m.count,f.start+f.count));for(let y=T,D=M;y<D;y+=3){let S=y,C=y+1,P=y+2;s=fo(this,p,t,i,c,h,u,S,C,P),s&&(s.faceIndex=Math.floor(y/3),s.face.materialIndex=m.materialIndex,e.push(s))}}else{let g=Math.max(0,f.start),x=Math.min(l.count,f.start+f.count);for(let m=g,p=x;m<p;m+=3){let T=m,M=m+1,y=m+2;s=fo(this,a,t,i,c,h,u,T,M,y),s&&(s.faceIndex=Math.floor(m/3),e.push(s))}}}};function bm(n,t,e,i,s,r,a,o){let l;if(t.side===je?l=i.intersectTriangle(a,r,s,!0,o):l=i.intersectTriangle(s,r,a,t.side===Qi,o),l===null)return null;uo.copy(o),uo.applyMatrix4(n.matrixWorld);let c=e.ray.origin.distanceTo(uo);return c<e.near||c>e.far?null:{distance:c,point:uo.clone(),object:n}}function fo(n,t,e,i,s,r,a,o,l,c){n.getVertexPosition(o,oo),n.getVertexPosition(l,lo),n.getVertexPosition(c,co);let h=bm(n,t,e,i,oo,lo,co,yd);if(h){let u=new I;Zi.getBarycoord(yd,oo,lo,co,u),s&&(h.uv=Zi.getInterpolatedAttribute(s,o,l,c,u,new j)),r&&(h.uv1=Zi.getInterpolatedAttribute(r,o,l,c,u,new j)),a&&(h.normal=Zi.getInterpolatedAttribute(a,o,l,c,u,new I),h.normal.dot(i.direction)>0&&h.normal.multiplyScalar(-1));let d={a:o,b:l,c,normal:new I,materialIndex:0};Zi.getNormal(oo,lo,co,d.normal),h.face=d,h.barycoord=u}return h}var Vn=class n extends tn{constructor(t=1,e=1,i=1,s=1,r=1,a=1){super(),this.type="BoxGeometry",this.parameters={width:t,height:e,depth:i,widthSegments:s,heightSegments:r,depthSegments:a};let o=this;s=Math.floor(s),r=Math.floor(r),a=Math.floor(a);let l=[],c=[],h=[],u=[],d=0,f=0;g("z","y","x",-1,-1,i,e,t,a,r,0),g("z","y","x",1,-1,i,e,-t,a,r,1),g("x","z","y",1,1,t,i,e,s,a,2),g("x","z","y",1,-1,t,i,-e,s,a,3),g("x","y","z",1,-1,t,e,i,s,r,4),g("x","y","z",-1,-1,t,e,-i,s,r,5),this.setIndex(l),this.setAttribute("position",new _e(c,3)),this.setAttribute("normal",new _e(h,3)),this.setAttribute("uv",new _e(u,2));function g(x,m,p,T,M,y,D,S,C,P,w){let _=y/C,R=D/P,G=y/2,N=D/2,z=S/2,J=C+1,V=P+1,it=0,W=0,ct=new I;for(let gt=0;gt<V;gt++){let Mt=gt*R-N;for(let Yt=0;Yt<J;Yt++){let he=Yt*_-G;ct[x]=he*T,ct[m]=Mt*M,ct[p]=z,c.push(ct.x,ct.y,ct.z),ct[x]=0,ct[m]=0,ct[p]=S>0?1:-1,h.push(ct.x,ct.y,ct.z),u.push(Yt/C),u.push(1-gt/P),it+=1}}for(let gt=0;gt<P;gt++)for(let Mt=0;Mt<C;Mt++){let Yt=d+Mt+J*gt,he=d+Mt+J*(gt+1),Y=d+(Mt+1)+J*(gt+1),rt=d+(Mt+1)+J*gt;l.push(Yt,he,rt),l.push(he,Y,rt),W+=6}o.addGroup(f,W,w),f+=W,d+=it}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new n(t.width,t.height,t.depth,t.widthSegments,t.heightSegments,t.depthSegments)}};function Pr(n){let t={};for(let e in n){t[e]={};for(let i in n[e]){let s=n[e][i];s&&(s.isColor||s.isMatrix3||s.isMatrix4||s.isVector2||s.isVector3||s.isVector4||s.isTexture||s.isQuaternion)?s.isRenderTargetTexture?(console.warn("UniformsUtils: Textures of render targets cannot be cloned via cloneUniforms() or mergeUniforms()."),t[e][i]=null):t[e][i]=s.clone():Array.isArray(s)?t[e][i]=s.slice():t[e][i]=s}}return t}function rn(n){let t={};for(let e=0;e<n.length;e++){let i=Pr(n[e]);for(let s in i)t[s]=i[s]}return t}function Mm(n){let t=[];for(let e=0;e<n.length;e++)t.push(n[e].clone());return t}function Mf(n){let t=n.getRenderTarget();return t===null?n.outputColorSpace:t.isXRRenderTarget===!0?t.texture.colorSpace:se.workingColorSpace}var ii={clone:Pr,merge:rn},Sm=`void main() {
	gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
}`,wm=`void main() {
	gl_FragColor = vec4( 1.0, 0.0, 0.0, 1.0 );
}`,Te=class extends es{static get type(){return"ShaderMaterial"}constructor(t){super(),this.isShaderMaterial=!0,this.defines={},this.uniforms={},this.uniformsGroups=[],this.vertexShader=Sm,this.fragmentShader=wm,this.linewidth=1,this.wireframe=!1,this.wireframeLinewidth=1,this.fog=!1,this.lights=!1,this.clipping=!1,this.forceSinglePass=!0,this.extensions={clipCullDistance:!1,multiDraw:!1},this.defaultAttributeValues={color:[1,1,1],uv:[0,0],uv1:[0,0]},this.index0AttributeName=void 0,this.uniformsNeedUpdate=!1,this.glslVersion=null,t!==void 0&&this.setValues(t)}copy(t){return super.copy(t),this.fragmentShader=t.fragmentShader,this.vertexShader=t.vertexShader,this.uniforms=Pr(t.uniforms),this.uniformsGroups=Mm(t.uniformsGroups),this.defines=Object.assign({},t.defines),this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.fog=t.fog,this.lights=t.lights,this.clipping=t.clipping,this.extensions=Object.assign({},t.extensions),this.glslVersion=t.glslVersion,this}toJSON(t){let e=super.toJSON(t);e.glslVersion=this.glslVersion,e.uniforms={};for(let s in this.uniforms){let a=this.uniforms[s].value;a&&a.isTexture?e.uniforms[s]={type:"t",value:a.toJSON(t).uuid}:a&&a.isColor?e.uniforms[s]={type:"c",value:a.getHex()}:a&&a.isVector2?e.uniforms[s]={type:"v2",value:a.toArray()}:a&&a.isVector3?e.uniforms[s]={type:"v3",value:a.toArray()}:a&&a.isVector4?e.uniforms[s]={type:"v4",value:a.toArray()}:a&&a.isMatrix3?e.uniforms[s]={type:"m3",value:a.toArray()}:a&&a.isMatrix4?e.uniforms[s]={type:"m4",value:a.toArray()}:e.uniforms[s]={value:a}}Object.keys(this.defines).length>0&&(e.defines=this.defines),e.vertexShader=this.vertexShader,e.fragmentShader=this.fragmentShader,e.lights=this.lights,e.clipping=this.clipping;let i={};for(let s in this.extensions)this.extensions[s]===!0&&(i[s]=!0);return Object.keys(i).length>0&&(e.extensions=i),e}},Bo=class extends ln{constructor(){super(),this.isCamera=!0,this.type="Camera",this.matrixWorldInverse=new pe,this.projectionMatrix=new pe,this.projectionMatrixInverse=new pe,this.coordinateSystem=wi}copy(t,e){return super.copy(t,e),this.matrixWorldInverse.copy(t.matrixWorldInverse),this.projectionMatrix.copy(t.projectionMatrix),this.projectionMatrixInverse.copy(t.projectionMatrixInverse),this.coordinateSystem=t.coordinateSystem,this}getWorldDirection(t){return super.getWorldDirection(t).negate()}updateMatrixWorld(t){super.updateMatrixWorld(t),this.matrixWorldInverse.copy(this.matrixWorld).invert()}updateWorldMatrix(t,e){super.updateWorldMatrix(t,e),this.matrixWorldInverse.copy(this.matrixWorld).invert()}clone(){return new this.constructor().copy(this)}},$i=new I,_d=new j,bd=new j,Qe=class extends Bo{constructor(t=50,e=1,i=.1,s=2e3){super(),this.isPerspectiveCamera=!0,this.type="PerspectiveCamera",this.fov=t,this.zoom=1,this.near=i,this.far=s,this.focus=10,this.aspect=e,this.view=null,this.filmGauge=35,this.filmOffset=0,this.updateProjectionMatrix()}copy(t,e){return super.copy(t,e),this.fov=t.fov,this.zoom=t.zoom,this.near=t.near,this.far=t.far,this.focus=t.focus,this.aspect=t.aspect,this.view=t.view===null?null:Object.assign({},t.view),this.filmGauge=t.filmGauge,this.filmOffset=t.filmOffset,this}setFocalLength(t){let e=.5*this.getFilmHeight()/t;this.fov=oh*2*Math.atan(e),this.updateProjectionMatrix()}getFocalLength(){let t=Math.tan(Hl*.5*this.fov);return .5*this.getFilmHeight()/t}getEffectiveFOV(){return oh*2*Math.atan(Math.tan(Hl*.5*this.fov)/this.zoom)}getFilmWidth(){return this.filmGauge*Math.min(this.aspect,1)}getFilmHeight(){return this.filmGauge/Math.max(this.aspect,1)}getViewBounds(t,e,i){$i.set(-1,-1,.5).applyMatrix4(this.projectionMatrixInverse),e.set($i.x,$i.y).multiplyScalar(-t/$i.z),$i.set(1,1,.5).applyMatrix4(this.projectionMatrixInverse),i.set($i.x,$i.y).multiplyScalar(-t/$i.z)}getViewSize(t,e){return this.getViewBounds(t,_d,bd),e.subVectors(bd,_d)}setViewOffset(t,e,i,s,r,a){this.aspect=t/e,this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=t,this.view.fullHeight=e,this.view.offsetX=i,this.view.offsetY=s,this.view.width=r,this.view.height=a,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){let t=this.near,e=t*Math.tan(Hl*.5*this.fov)/this.zoom,i=2*e,s=this.aspect*i,r=-.5*s,a=this.view;if(this.view!==null&&this.view.enabled){let l=a.fullWidth,c=a.fullHeight;r+=a.offsetX*s/l,e-=a.offsetY*i/c,s*=a.width/l,i*=a.height/c}let o=this.filmOffset;o!==0&&(r+=t*o/this.getFilmWidth()),this.projectionMatrix.makePerspective(r,r+s,e,e-i,t,this.far,this.coordinateSystem),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(t){let e=super.toJSON(t);return e.object.fov=this.fov,e.object.zoom=this.zoom,e.object.near=this.near,e.object.far=this.far,e.object.focus=this.focus,e.object.aspect=this.aspect,this.view!==null&&(e.object.view=Object.assign({},this.view)),e.object.filmGauge=this.filmGauge,e.object.filmOffset=this.filmOffset,e}},ur=-90,dr=1,dh=class extends ln{constructor(t,e,i){super(),this.type="CubeCamera",this.renderTarget=i,this.coordinateSystem=null,this.activeMipmapLevel=0;let s=new Qe(ur,dr,t,e);s.layers=this.layers,this.add(s);let r=new Qe(ur,dr,t,e);r.layers=this.layers,this.add(r);let a=new Qe(ur,dr,t,e);a.layers=this.layers,this.add(a);let o=new Qe(ur,dr,t,e);o.layers=this.layers,this.add(o);let l=new Qe(ur,dr,t,e);l.layers=this.layers,this.add(l);let c=new Qe(ur,dr,t,e);c.layers=this.layers,this.add(c)}updateCoordinateSystem(){let t=this.coordinateSystem,e=this.children.concat(),[i,s,r,a,o,l]=e;for(let c of e)this.remove(c);if(t===wi)i.up.set(0,1,0),i.lookAt(1,0,0),s.up.set(0,1,0),s.lookAt(-1,0,0),r.up.set(0,0,-1),r.lookAt(0,1,0),a.up.set(0,0,1),a.lookAt(0,-1,0),o.up.set(0,1,0),o.lookAt(0,0,1),l.up.set(0,1,0),l.lookAt(0,0,-1);else if(t===Fo)i.up.set(0,-1,0),i.lookAt(-1,0,0),s.up.set(0,-1,0),s.lookAt(1,0,0),r.up.set(0,0,1),r.lookAt(0,1,0),a.up.set(0,0,-1),a.lookAt(0,-1,0),o.up.set(0,-1,0),o.lookAt(0,0,1),l.up.set(0,-1,0),l.lookAt(0,0,-1);else throw new Error("THREE.CubeCamera.updateCoordinateSystem(): Invalid coordinate system: "+t);for(let c of e)this.add(c),c.updateMatrixWorld()}update(t,e){this.parent===null&&this.updateMatrixWorld();let{renderTarget:i,activeMipmapLevel:s}=this;this.coordinateSystem!==t.coordinateSystem&&(this.coordinateSystem=t.coordinateSystem,this.updateCoordinateSystem());let[r,a,o,l,c,h]=this.children,u=t.getRenderTarget(),d=t.getActiveCubeFace(),f=t.getActiveMipmapLevel(),g=t.xr.enabled;t.xr.enabled=!1;let x=i.texture.generateMipmaps;i.texture.generateMipmaps=!1,t.setRenderTarget(i,0,s),t.render(e,r),t.setRenderTarget(i,1,s),t.render(e,a),t.setRenderTarget(i,2,s),t.render(e,o),t.setRenderTarget(i,3,s),t.render(e,l),t.setRenderTarget(i,4,s),t.render(e,c),i.texture.generateMipmaps=x,t.setRenderTarget(i,5,s),t.render(e,h),t.setRenderTarget(u,d,f),t.xr.enabled=g,i.texture.needsPMREMUpdate=!0}},Oo=class extends on{constructor(t,e,i,s,r,a,o,l,c,h){t=t!==void 0?t:[],e=e!==void 0?e:Tr,super(t,e,i,s,r,a,o,l,c,h),this.isCubeTexture=!0,this.flipY=!1}get images(){return this.image}set images(t){this.image=t}},fh=class extends Ue{constructor(t=1,e={}){super(t,t,e),this.isWebGLCubeRenderTarget=!0;let i={width:t,height:t,depth:1},s=[i,i,i,i,i,i];this.texture=new Oo(s,e.mapping,e.wrapS,e.wrapT,e.magFilter,e.minFilter,e.format,e.type,e.anisotropy,e.colorSpace),this.texture.isRenderTargetTexture=!0,this.texture.generateMipmaps=e.generateMipmaps!==void 0?e.generateMipmaps:!1,this.texture.minFilter=e.minFilter!==void 0?e.minFilter:Jn}fromEquirectangularTexture(t,e){this.texture.type=e.type,this.texture.colorSpace=e.colorSpace,this.texture.generateMipmaps=e.generateMipmaps,this.texture.minFilter=e.minFilter,this.texture.magFilter=e.magFilter;let i={uniforms:{tEquirect:{value:null}},vertexShader:`

				varying vec3 vWorldDirection;

				vec3 transformDirection( in vec3 dir, in mat4 matrix ) {

					return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );

				}

				void main() {

					vWorldDirection = transformDirection( position, modelMatrix );

					#include <begin_vertex>
					#include <project_vertex>

				}
			`,fragmentShader:`

				uniform sampler2D tEquirect;

				varying vec3 vWorldDirection;

				#include <common>

				void main() {

					vec3 direction = normalize( vWorldDirection );

					vec2 sampleUV = equirectUv( direction );

					gl_FragColor = texture2D( tEquirect, sampleUV );

				}
			`},s=new Vn(5,5,5),r=new Te({name:"CubemapFromEquirect",uniforms:Pr(i.uniforms),vertexShader:i.vertexShader,fragmentShader:i.fragmentShader,side:je,blending:Rn});r.uniforms.tEquirect.value=e;let a=new Ot(s,r),o=e.minFilter;return e.minFilter===Ls&&(e.minFilter=Jn),new dh(1,10,this).update(t,a),e.minFilter=o,a.geometry.dispose(),a.material.dispose(),this}clear(t,e,i,s){let r=t.getRenderTarget();for(let a=0;a<6;a++)t.setRenderTarget(this,a),t.clear(e,i,s);t.setRenderTarget(r)}},lc=new I,Em=new I,Tm=new Kt,Mi=class{constructor(t=new I(1,0,0),e=0){this.isPlane=!0,this.normal=t,this.constant=e}set(t,e){return this.normal.copy(t),this.constant=e,this}setComponents(t,e,i,s){return this.normal.set(t,e,i),this.constant=s,this}setFromNormalAndCoplanarPoint(t,e){return this.normal.copy(t),this.constant=-e.dot(this.normal),this}setFromCoplanarPoints(t,e,i){let s=lc.subVectors(i,e).cross(Em.subVectors(t,e)).normalize();return this.setFromNormalAndCoplanarPoint(s,t),this}copy(t){return this.normal.copy(t.normal),this.constant=t.constant,this}normalize(){let t=1/this.normal.length();return this.normal.multiplyScalar(t),this.constant*=t,this}negate(){return this.constant*=-1,this.normal.negate(),this}distanceToPoint(t){return this.normal.dot(t)+this.constant}distanceToSphere(t){return this.distanceToPoint(t.center)-t.radius}projectPoint(t,e){return e.copy(t).addScaledVector(this.normal,-this.distanceToPoint(t))}intersectLine(t,e){let i=t.delta(lc),s=this.normal.dot(i);if(s===0)return this.distanceToPoint(t.start)===0?e.copy(t.start):null;let r=-(t.start.dot(this.normal)+this.constant)/s;return r<0||r>1?null:e.copy(t.start).addScaledVector(i,r)}intersectsLine(t){let e=this.distanceToPoint(t.start),i=this.distanceToPoint(t.end);return e<0&&i>0||i<0&&e>0}intersectsBox(t){return t.intersectsPlane(this)}intersectsSphere(t){return t.intersectsPlane(this)}coplanarPoint(t){return t.copy(this.normal).multiplyScalar(-this.constant)}applyMatrix4(t,e){let i=e||Tm.getNormalMatrix(t),s=this.coplanarPoint(lc).applyMatrix4(t),r=this.normal.applyMatrix3(i).normalize();return this.constant=-s.dot(r),this}translate(t){return this.constant-=t.dot(this.normal),this}equals(t){return t.normal.equals(this.normal)&&t.constant===this.constant}clone(){return new this.constructor().copy(this)}},Rs=new Ns,po=new I,pa=class{constructor(t=new Mi,e=new Mi,i=new Mi,s=new Mi,r=new Mi,a=new Mi){this.planes=[t,e,i,s,r,a]}set(t,e,i,s,r,a){let o=this.planes;return o[0].copy(t),o[1].copy(e),o[2].copy(i),o[3].copy(s),o[4].copy(r),o[5].copy(a),this}copy(t){let e=this.planes;for(let i=0;i<6;i++)e[i].copy(t.planes[i]);return this}setFromProjectionMatrix(t,e=wi){let i=this.planes,s=t.elements,r=s[0],a=s[1],o=s[2],l=s[3],c=s[4],h=s[5],u=s[6],d=s[7],f=s[8],g=s[9],x=s[10],m=s[11],p=s[12],T=s[13],M=s[14],y=s[15];if(i[0].setComponents(l-r,d-c,m-f,y-p).normalize(),i[1].setComponents(l+r,d+c,m+f,y+p).normalize(),i[2].setComponents(l+a,d+h,m+g,y+T).normalize(),i[3].setComponents(l-a,d-h,m-g,y-T).normalize(),i[4].setComponents(l-o,d-u,m-x,y-M).normalize(),e===wi)i[5].setComponents(l+o,d+u,m+x,y+M).normalize();else if(e===Fo)i[5].setComponents(o,u,x,M).normalize();else throw new Error("THREE.Frustum.setFromProjectionMatrix(): Invalid coordinate system: "+e);return this}intersectsObject(t){if(t.boundingSphere!==void 0)t.boundingSphere===null&&t.computeBoundingSphere(),Rs.copy(t.boundingSphere).applyMatrix4(t.matrixWorld);else{let e=t.geometry;e.boundingSphere===null&&e.computeBoundingSphere(),Rs.copy(e.boundingSphere).applyMatrix4(t.matrixWorld)}return this.intersectsSphere(Rs)}intersectsSprite(t){return Rs.center.set(0,0,0),Rs.radius=.7071067811865476,Rs.applyMatrix4(t.matrixWorld),this.intersectsSphere(Rs)}intersectsSphere(t){let e=this.planes,i=t.center,s=-t.radius;for(let r=0;r<6;r++)if(e[r].distanceToPoint(i)<s)return!1;return!0}intersectsBox(t){let e=this.planes;for(let i=0;i<6;i++){let s=e[i];if(po.x=s.normal.x>0?t.max.x:t.min.x,po.y=s.normal.y>0?t.max.y:t.min.y,po.z=s.normal.z>0?t.max.z:t.min.z,s.distanceToPoint(po)<0)return!1}return!0}containsPoint(t){let e=this.planes;for(let i=0;i<6;i++)if(e[i].distanceToPoint(t)<0)return!1;return!0}clone(){return new this.constructor().copy(this)}};function Sf(){let n=null,t=!1,e=null,i=null;function s(r,a){e(r,a),i=n.requestAnimationFrame(s)}return{start:function(){t!==!0&&e!==null&&(i=n.requestAnimationFrame(s),t=!0)},stop:function(){n.cancelAnimationFrame(i),t=!1},setAnimationLoop:function(r){e=r},setContext:function(r){n=r}}}function Am(n){let t=new WeakMap;function e(o,l){let c=o.array,h=o.usage,u=c.byteLength,d=n.createBuffer();n.bindBuffer(l,d),n.bufferData(l,c,h),o.onUploadCallback();let f;if(c instanceof Float32Array)f=n.FLOAT;else if(c instanceof Uint16Array)o.isFloat16BufferAttribute?f=n.HALF_FLOAT:f=n.UNSIGNED_SHORT;else if(c instanceof Int16Array)f=n.SHORT;else if(c instanceof Uint32Array)f=n.UNSIGNED_INT;else if(c instanceof Int32Array)f=n.INT;else if(c instanceof Int8Array)f=n.BYTE;else if(c instanceof Uint8Array)f=n.UNSIGNED_BYTE;else if(c instanceof Uint8ClampedArray)f=n.UNSIGNED_BYTE;else throw new Error("THREE.WebGLAttributes: Unsupported buffer data format: "+c);return{buffer:d,type:f,bytesPerElement:c.BYTES_PER_ELEMENT,version:o.version,size:u}}function i(o,l,c){let h=l.array,u=l.updateRanges;if(n.bindBuffer(c,o),u.length===0)n.bufferSubData(c,0,h);else{u.sort((f,g)=>f.start-g.start);let d=0;for(let f=1;f<u.length;f++){let g=u[d],x=u[f];x.start<=g.start+g.count+1?g.count=Math.max(g.count,x.start+x.count-g.start):(++d,u[d]=x)}u.length=d+1;for(let f=0,g=u.length;f<g;f++){let x=u[f];n.bufferSubData(c,x.start*h.BYTES_PER_ELEMENT,h,x.start,x.count)}l.clearUpdateRanges()}l.onUploadCallback()}function s(o){return o.isInterleavedBufferAttribute&&(o=o.data),t.get(o)}function r(o){o.isInterleavedBufferAttribute&&(o=o.data);let l=t.get(o);l&&(n.deleteBuffer(l.buffer),t.delete(o))}function a(o,l){if(o.isInterleavedBufferAttribute&&(o=o.data),o.isGLBufferAttribute){let h=t.get(o);(!h||h.version<o.version)&&t.set(o,{buffer:o.buffer,type:o.type,bytesPerElement:o.elementSize,version:o.version});return}let c=t.get(o);if(c===void 0)t.set(o,e(o,l));else if(c.version<o.version){if(c.size!==o.array.byteLength)throw new Error("THREE.WebGLAttributes: The size of the buffer attribute's array buffer does not match the original size. Resizing buffer attributes is not supported.");i(c.buffer,o,l),c.version=o.version}}return{get:s,remove:r,update:a}}var ei=class n extends tn{constructor(t=1,e=1,i=1,s=1){super(),this.type="PlaneGeometry",this.parameters={width:t,height:e,widthSegments:i,heightSegments:s};let r=t/2,a=e/2,o=Math.floor(i),l=Math.floor(s),c=o+1,h=l+1,u=t/o,d=e/l,f=[],g=[],x=[],m=[];for(let p=0;p<h;p++){let T=p*d-a;for(let M=0;M<c;M++){let y=M*u-r;g.push(y,-T,0),x.push(0,0,1),m.push(M/o),m.push(1-p/l)}}for(let p=0;p<l;p++)for(let T=0;T<o;T++){let M=T+c*p,y=T+c*(p+1),D=T+1+c*(p+1),S=T+1+c*p;f.push(M,y,S),f.push(y,D,S)}this.setIndex(f),this.setAttribute("position",new _e(g,3)),this.setAttribute("normal",new _e(x,3)),this.setAttribute("uv",new _e(m,2))}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new n(t.width,t.height,t.widthSegments,t.heightSegments)}},Cm=`#ifdef USE_ALPHAHASH
	if ( diffuseColor.a < getAlphaHashThreshold( vPosition ) ) discard;
#endif`,Rm=`#ifdef USE_ALPHAHASH
	const float ALPHA_HASH_SCALE = 0.05;
	float hash2D( vec2 value ) {
		return fract( 1.0e4 * sin( 17.0 * value.x + 0.1 * value.y ) * ( 0.1 + abs( sin( 13.0 * value.y + value.x ) ) ) );
	}
	float hash3D( vec3 value ) {
		return hash2D( vec2( hash2D( value.xy ), value.z ) );
	}
	float getAlphaHashThreshold( vec3 position ) {
		float maxDeriv = max(
			length( dFdx( position.xyz ) ),
			length( dFdy( position.xyz ) )
		);
		float pixScale = 1.0 / ( ALPHA_HASH_SCALE * maxDeriv );
		vec2 pixScales = vec2(
			exp2( floor( log2( pixScale ) ) ),
			exp2( ceil( log2( pixScale ) ) )
		);
		vec2 alpha = vec2(
			hash3D( floor( pixScales.x * position.xyz ) ),
			hash3D( floor( pixScales.y * position.xyz ) )
		);
		float lerpFactor = fract( log2( pixScale ) );
		float x = ( 1.0 - lerpFactor ) * alpha.x + lerpFactor * alpha.y;
		float a = min( lerpFactor, 1.0 - lerpFactor );
		vec3 cases = vec3(
			x * x / ( 2.0 * a * ( 1.0 - a ) ),
			( x - 0.5 * a ) / ( 1.0 - a ),
			1.0 - ( ( 1.0 - x ) * ( 1.0 - x ) / ( 2.0 * a * ( 1.0 - a ) ) )
		);
		float threshold = ( x < ( 1.0 - a ) )
			? ( ( x < a ) ? cases.x : cases.y )
			: cases.z;
		return clamp( threshold , 1.0e-6, 1.0 );
	}
#endif`,Pm=`#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, vAlphaMapUv ).g;
#endif`,Im=`#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,Fm=`#ifdef USE_ALPHATEST
	#ifdef ALPHA_TO_COVERAGE
	diffuseColor.a = smoothstep( alphaTest, alphaTest + fwidth( diffuseColor.a ), diffuseColor.a );
	if ( diffuseColor.a == 0.0 ) discard;
	#else
	if ( diffuseColor.a < alphaTest ) discard;
	#endif
#endif`,Dm=`#ifdef USE_ALPHATEST
	uniform float alphaTest;
#endif`,km=`#ifdef USE_AOMAP
	float ambientOcclusion = ( texture2D( aoMap, vAoMapUv ).r - 1.0 ) * aoMapIntensity + 1.0;
	reflectedLight.indirectDiffuse *= ambientOcclusion;
	#if defined( USE_CLEARCOAT ) 
		clearcoatSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_SHEEN ) 
		sheenSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_ENVMAP ) && defined( STANDARD )
		float dotNV = saturate( dot( geometryNormal, geometryViewDir ) );
		reflectedLight.indirectSpecular *= computeSpecularOcclusion( dotNV, ambientOcclusion, material.roughness );
	#endif
#endif`,Lm=`#ifdef USE_AOMAP
	uniform sampler2D aoMap;
	uniform float aoMapIntensity;
#endif`,Um=`#ifdef USE_BATCHING
	#if ! defined( GL_ANGLE_multi_draw )
	#define gl_DrawID _gl_DrawID
	uniform int _gl_DrawID;
	#endif
	uniform highp sampler2D batchingTexture;
	uniform highp usampler2D batchingIdTexture;
	mat4 getBatchingMatrix( const in float i ) {
		int size = textureSize( batchingTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( batchingTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( batchingTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( batchingTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( batchingTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
	float getIndirectIndex( const in int i ) {
		int size = textureSize( batchingIdTexture, 0 ).x;
		int x = i % size;
		int y = i / size;
		return float( texelFetch( batchingIdTexture, ivec2( x, y ), 0 ).r );
	}
#endif
#ifdef USE_BATCHING_COLOR
	uniform sampler2D batchingColorTexture;
	vec3 getBatchingColor( const in float i ) {
		int size = textureSize( batchingColorTexture, 0 ).x;
		int j = int( i );
		int x = j % size;
		int y = j / size;
		return texelFetch( batchingColorTexture, ivec2( x, y ), 0 ).rgb;
	}
#endif`,Nm=`#ifdef USE_BATCHING
	mat4 batchingMatrix = getBatchingMatrix( getIndirectIndex( gl_DrawID ) );
#endif`,Bm=`vec3 transformed = vec3( position );
#ifdef USE_ALPHAHASH
	vPosition = vec3( position );
#endif`,Om=`vec3 objectNormal = vec3( normal );
#ifdef USE_TANGENT
	vec3 objectTangent = vec3( tangent.xyz );
#endif`,zm=`float G_BlinnPhong_Implicit( ) {
	return 0.25;
}
float D_BlinnPhong( const in float shininess, const in float dotNH ) {
	return RECIPROCAL_PI * ( shininess * 0.5 + 1.0 ) * pow( dotNH, shininess );
}
vec3 BRDF_BlinnPhong( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in vec3 specularColor, const in float shininess ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( specularColor, 1.0, dotVH );
	float G = G_BlinnPhong_Implicit( );
	float D = D_BlinnPhong( shininess, dotNH );
	return F * ( G * D );
} // validated`,Hm=`#ifdef USE_IRIDESCENCE
	const mat3 XYZ_TO_REC709 = mat3(
		 3.2404542, -0.9692660,  0.0556434,
		-1.5371385,  1.8760108, -0.2040259,
		-0.4985314,  0.0415560,  1.0572252
	);
	vec3 Fresnel0ToIor( vec3 fresnel0 ) {
		vec3 sqrtF0 = sqrt( fresnel0 );
		return ( vec3( 1.0 ) + sqrtF0 ) / ( vec3( 1.0 ) - sqrtF0 );
	}
	vec3 IorToFresnel0( vec3 transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - vec3( incidentIor ) ) / ( transmittedIor + vec3( incidentIor ) ) );
	}
	float IorToFresnel0( float transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - incidentIor ) / ( transmittedIor + incidentIor ));
	}
	vec3 evalSensitivity( float OPD, vec3 shift ) {
		float phase = 2.0 * PI * OPD * 1.0e-9;
		vec3 val = vec3( 5.4856e-13, 4.4201e-13, 5.2481e-13 );
		vec3 pos = vec3( 1.6810e+06, 1.7953e+06, 2.2084e+06 );
		vec3 var = vec3( 4.3278e+09, 9.3046e+09, 6.6121e+09 );
		vec3 xyz = val * sqrt( 2.0 * PI * var ) * cos( pos * phase + shift ) * exp( - pow2( phase ) * var );
		xyz.x += 9.7470e-14 * sqrt( 2.0 * PI * 4.5282e+09 ) * cos( 2.2399e+06 * phase + shift[ 0 ] ) * exp( - 4.5282e+09 * pow2( phase ) );
		xyz /= 1.0685e-7;
		vec3 rgb = XYZ_TO_REC709 * xyz;
		return rgb;
	}
	vec3 evalIridescence( float outsideIOR, float eta2, float cosTheta1, float thinFilmThickness, vec3 baseF0 ) {
		vec3 I;
		float iridescenceIOR = mix( outsideIOR, eta2, smoothstep( 0.0, 0.03, thinFilmThickness ) );
		float sinTheta2Sq = pow2( outsideIOR / iridescenceIOR ) * ( 1.0 - pow2( cosTheta1 ) );
		float cosTheta2Sq = 1.0 - sinTheta2Sq;
		if ( cosTheta2Sq < 0.0 ) {
			return vec3( 1.0 );
		}
		float cosTheta2 = sqrt( cosTheta2Sq );
		float R0 = IorToFresnel0( iridescenceIOR, outsideIOR );
		float R12 = F_Schlick( R0, 1.0, cosTheta1 );
		float T121 = 1.0 - R12;
		float phi12 = 0.0;
		if ( iridescenceIOR < outsideIOR ) phi12 = PI;
		float phi21 = PI - phi12;
		vec3 baseIOR = Fresnel0ToIor( clamp( baseF0, 0.0, 0.9999 ) );		vec3 R1 = IorToFresnel0( baseIOR, iridescenceIOR );
		vec3 R23 = F_Schlick( R1, 1.0, cosTheta2 );
		vec3 phi23 = vec3( 0.0 );
		if ( baseIOR[ 0 ] < iridescenceIOR ) phi23[ 0 ] = PI;
		if ( baseIOR[ 1 ] < iridescenceIOR ) phi23[ 1 ] = PI;
		if ( baseIOR[ 2 ] < iridescenceIOR ) phi23[ 2 ] = PI;
		float OPD = 2.0 * iridescenceIOR * thinFilmThickness * cosTheta2;
		vec3 phi = vec3( phi21 ) + phi23;
		vec3 R123 = clamp( R12 * R23, 1e-5, 0.9999 );
		vec3 r123 = sqrt( R123 );
		vec3 Rs = pow2( T121 ) * R23 / ( vec3( 1.0 ) - R123 );
		vec3 C0 = R12 + Rs;
		I = C0;
		vec3 Cm = Rs - T121;
		for ( int m = 1; m <= 2; ++ m ) {
			Cm *= r123;
			vec3 Sm = 2.0 * evalSensitivity( float( m ) * OPD, float( m ) * phi );
			I += Cm * Sm;
		}
		return max( I, vec3( 0.0 ) );
	}
#endif`,Vm=`#ifdef USE_BUMPMAP
	uniform sampler2D bumpMap;
	uniform float bumpScale;
	vec2 dHdxy_fwd() {
		vec2 dSTdx = dFdx( vBumpMapUv );
		vec2 dSTdy = dFdy( vBumpMapUv );
		float Hll = bumpScale * texture2D( bumpMap, vBumpMapUv ).x;
		float dBx = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdx ).x - Hll;
		float dBy = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdy ).x - Hll;
		return vec2( dBx, dBy );
	}
	vec3 perturbNormalArb( vec3 surf_pos, vec3 surf_norm, vec2 dHdxy, float faceDirection ) {
		vec3 vSigmaX = normalize( dFdx( surf_pos.xyz ) );
		vec3 vSigmaY = normalize( dFdy( surf_pos.xyz ) );
		vec3 vN = surf_norm;
		vec3 R1 = cross( vSigmaY, vN );
		vec3 R2 = cross( vN, vSigmaX );
		float fDet = dot( vSigmaX, R1 ) * faceDirection;
		vec3 vGrad = sign( fDet ) * ( dHdxy.x * R1 + dHdxy.y * R2 );
		return normalize( abs( fDet ) * surf_norm - vGrad );
	}
#endif`,Gm=`#if NUM_CLIPPING_PLANES > 0
	vec4 plane;
	#ifdef ALPHA_TO_COVERAGE
		float distanceToPlane, distanceGradient;
		float clipOpacity = 1.0;
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
			distanceGradient = fwidth( distanceToPlane ) / 2.0;
			clipOpacity *= smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			if ( clipOpacity == 0.0 ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			float unionClipOpacity = 1.0;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
				distanceGradient = fwidth( distanceToPlane ) / 2.0;
				unionClipOpacity *= 1.0 - smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			}
			#pragma unroll_loop_end
			clipOpacity *= 1.0 - unionClipOpacity;
		#endif
		diffuseColor.a *= clipOpacity;
		if ( diffuseColor.a == 0.0 ) discard;
	#else
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			if ( dot( vClipPosition, plane.xyz ) > plane.w ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			bool clipped = true;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				clipped = ( dot( vClipPosition, plane.xyz ) > plane.w ) && clipped;
			}
			#pragma unroll_loop_end
			if ( clipped ) discard;
		#endif
	#endif
#endif`,Wm=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
	uniform vec4 clippingPlanes[ NUM_CLIPPING_PLANES ];
#endif`,qm=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
#endif`,Xm=`#if NUM_CLIPPING_PLANES > 0
	vClipPosition = - mvPosition.xyz;
#endif`,$m=`#if defined( USE_COLOR_ALPHA )
	diffuseColor *= vColor;
#elif defined( USE_COLOR )
	diffuseColor.rgb *= vColor;
#endif`,Ym=`#if defined( USE_COLOR_ALPHA )
	varying vec4 vColor;
#elif defined( USE_COLOR )
	varying vec3 vColor;
#endif`,Zm=`#if defined( USE_COLOR_ALPHA )
	varying vec4 vColor;
#elif defined( USE_COLOR ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	varying vec3 vColor;
#endif`,Jm=`#if defined( USE_COLOR_ALPHA )
	vColor = vec4( 1.0 );
#elif defined( USE_COLOR ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	vColor = vec3( 1.0 );
#endif
#ifdef USE_COLOR
	vColor *= color;
#endif
#ifdef USE_INSTANCING_COLOR
	vColor.xyz *= instanceColor.xyz;
#endif
#ifdef USE_BATCHING_COLOR
	vec3 batchingColor = getBatchingColor( getIndirectIndex( gl_DrawID ) );
	vColor.xyz *= batchingColor.xyz;
#endif`,Km=`#define PI 3.141592653589793
#define PI2 6.283185307179586
#define PI_HALF 1.5707963267948966
#define RECIPROCAL_PI 0.3183098861837907
#define RECIPROCAL_PI2 0.15915494309189535
#define EPSILON 1e-6
#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
#define whiteComplement( a ) ( 1.0 - saturate( a ) )
float pow2( const in float x ) { return x*x; }
vec3 pow2( const in vec3 x ) { return x*x; }
float pow3( const in float x ) { return x*x*x; }
float pow4( const in float x ) { float x2 = x*x; return x2*x2; }
float max3( const in vec3 v ) { return max( max( v.x, v.y ), v.z ); }
float average( const in vec3 v ) { return dot( v, vec3( 0.3333333 ) ); }
highp float rand( const in vec2 uv ) {
	const highp float a = 12.9898, b = 78.233, c = 43758.5453;
	highp float dt = dot( uv.xy, vec2( a,b ) ), sn = mod( dt, PI );
	return fract( sin( sn ) * c );
}
#ifdef HIGH_PRECISION
	float precisionSafeLength( vec3 v ) { return length( v ); }
#else
	float precisionSafeLength( vec3 v ) {
		float maxComponent = max3( abs( v ) );
		return length( v / maxComponent ) * maxComponent;
	}
#endif
struct IncidentLight {
	vec3 color;
	vec3 direction;
	bool visible;
};
struct ReflectedLight {
	vec3 directDiffuse;
	vec3 directSpecular;
	vec3 indirectDiffuse;
	vec3 indirectSpecular;
};
#ifdef USE_ALPHAHASH
	varying vec3 vPosition;
#endif
vec3 transformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );
}
vec3 inverseTransformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( vec4( dir, 0.0 ) * matrix ).xyz );
}
mat3 transposeMat3( const in mat3 m ) {
	mat3 tmp;
	tmp[ 0 ] = vec3( m[ 0 ].x, m[ 1 ].x, m[ 2 ].x );
	tmp[ 1 ] = vec3( m[ 0 ].y, m[ 1 ].y, m[ 2 ].y );
	tmp[ 2 ] = vec3( m[ 0 ].z, m[ 1 ].z, m[ 2 ].z );
	return tmp;
}
bool isPerspectiveMatrix( mat4 m ) {
	return m[ 2 ][ 3 ] == - 1.0;
}
vec2 equirectUv( in vec3 dir ) {
	float u = atan( dir.z, dir.x ) * RECIPROCAL_PI2 + 0.5;
	float v = asin( clamp( dir.y, - 1.0, 1.0 ) ) * RECIPROCAL_PI + 0.5;
	return vec2( u, v );
}
vec3 BRDF_Lambert( const in vec3 diffuseColor ) {
	return RECIPROCAL_PI * diffuseColor;
}
vec3 F_Schlick( const in vec3 f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
}
float F_Schlick( const in float f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
} // validated`,Qm=`#ifdef ENVMAP_TYPE_CUBE_UV
	#define cubeUV_minMipLevel 4.0
	#define cubeUV_minTileSize 16.0
	float getFace( vec3 direction ) {
		vec3 absDirection = abs( direction );
		float face = - 1.0;
		if ( absDirection.x > absDirection.z ) {
			if ( absDirection.x > absDirection.y )
				face = direction.x > 0.0 ? 0.0 : 3.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		} else {
			if ( absDirection.z > absDirection.y )
				face = direction.z > 0.0 ? 2.0 : 5.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		}
		return face;
	}
	vec2 getUV( vec3 direction, float face ) {
		vec2 uv;
		if ( face == 0.0 ) {
			uv = vec2( direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 1.0 ) {
			uv = vec2( - direction.x, - direction.z ) / abs( direction.y );
		} else if ( face == 2.0 ) {
			uv = vec2( - direction.x, direction.y ) / abs( direction.z );
		} else if ( face == 3.0 ) {
			uv = vec2( - direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 4.0 ) {
			uv = vec2( - direction.x, direction.z ) / abs( direction.y );
		} else {
			uv = vec2( direction.x, direction.y ) / abs( direction.z );
		}
		return 0.5 * ( uv + 1.0 );
	}
	vec3 bilinearCubeUV( sampler2D envMap, vec3 direction, float mipInt ) {
		float face = getFace( direction );
		float filterInt = max( cubeUV_minMipLevel - mipInt, 0.0 );
		mipInt = max( mipInt, cubeUV_minMipLevel );
		float faceSize = exp2( mipInt );
		highp vec2 uv = getUV( direction, face ) * ( faceSize - 2.0 ) + 1.0;
		if ( face > 2.0 ) {
			uv.y += faceSize;
			face -= 3.0;
		}
		uv.x += face * faceSize;
		uv.x += filterInt * 3.0 * cubeUV_minTileSize;
		uv.y += 4.0 * ( exp2( CUBEUV_MAX_MIP ) - faceSize );
		uv.x *= CUBEUV_TEXEL_WIDTH;
		uv.y *= CUBEUV_TEXEL_HEIGHT;
		#ifdef texture2DGradEXT
			return texture2DGradEXT( envMap, uv, vec2( 0.0 ), vec2( 0.0 ) ).rgb;
		#else
			return texture2D( envMap, uv ).rgb;
		#endif
	}
	#define cubeUV_r0 1.0
	#define cubeUV_m0 - 2.0
	#define cubeUV_r1 0.8
	#define cubeUV_m1 - 1.0
	#define cubeUV_r4 0.4
	#define cubeUV_m4 2.0
	#define cubeUV_r5 0.305
	#define cubeUV_m5 3.0
	#define cubeUV_r6 0.21
	#define cubeUV_m6 4.0
	float roughnessToMip( float roughness ) {
		float mip = 0.0;
		if ( roughness >= cubeUV_r1 ) {
			mip = ( cubeUV_r0 - roughness ) * ( cubeUV_m1 - cubeUV_m0 ) / ( cubeUV_r0 - cubeUV_r1 ) + cubeUV_m0;
		} else if ( roughness >= cubeUV_r4 ) {
			mip = ( cubeUV_r1 - roughness ) * ( cubeUV_m4 - cubeUV_m1 ) / ( cubeUV_r1 - cubeUV_r4 ) + cubeUV_m1;
		} else if ( roughness >= cubeUV_r5 ) {
			mip = ( cubeUV_r4 - roughness ) * ( cubeUV_m5 - cubeUV_m4 ) / ( cubeUV_r4 - cubeUV_r5 ) + cubeUV_m4;
		} else if ( roughness >= cubeUV_r6 ) {
			mip = ( cubeUV_r5 - roughness ) * ( cubeUV_m6 - cubeUV_m5 ) / ( cubeUV_r5 - cubeUV_r6 ) + cubeUV_m5;
		} else {
			mip = - 2.0 * log2( 1.16 * roughness );		}
		return mip;
	}
	vec4 textureCubeUV( sampler2D envMap, vec3 sampleDir, float roughness ) {
		float mip = clamp( roughnessToMip( roughness ), cubeUV_m0, CUBEUV_MAX_MIP );
		float mipF = fract( mip );
		float mipInt = floor( mip );
		vec3 color0 = bilinearCubeUV( envMap, sampleDir, mipInt );
		if ( mipF == 0.0 ) {
			return vec4( color0, 1.0 );
		} else {
			vec3 color1 = bilinearCubeUV( envMap, sampleDir, mipInt + 1.0 );
			return vec4( mix( color0, color1, mipF ), 1.0 );
		}
	}
#endif`,jm=`vec3 transformedNormal = objectNormal;
#ifdef USE_TANGENT
	vec3 transformedTangent = objectTangent;
#endif
#ifdef USE_BATCHING
	mat3 bm = mat3( batchingMatrix );
	transformedNormal /= vec3( dot( bm[ 0 ], bm[ 0 ] ), dot( bm[ 1 ], bm[ 1 ] ), dot( bm[ 2 ], bm[ 2 ] ) );
	transformedNormal = bm * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = bm * transformedTangent;
	#endif
#endif
#ifdef USE_INSTANCING
	mat3 im = mat3( instanceMatrix );
	transformedNormal /= vec3( dot( im[ 0 ], im[ 0 ] ), dot( im[ 1 ], im[ 1 ] ), dot( im[ 2 ], im[ 2 ] ) );
	transformedNormal = im * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = im * transformedTangent;
	#endif
#endif
transformedNormal = normalMatrix * transformedNormal;
#ifdef FLIP_SIDED
	transformedNormal = - transformedNormal;
#endif
#ifdef USE_TANGENT
	transformedTangent = ( modelViewMatrix * vec4( transformedTangent, 0.0 ) ).xyz;
	#ifdef FLIP_SIDED
		transformedTangent = - transformedTangent;
	#endif
#endif`,t0=`#ifdef USE_DISPLACEMENTMAP
	uniform sampler2D displacementMap;
	uniform float displacementScale;
	uniform float displacementBias;
#endif`,e0=`#ifdef USE_DISPLACEMENTMAP
	transformed += normalize( objectNormal ) * ( texture2D( displacementMap, vDisplacementMapUv ).x * displacementScale + displacementBias );
#endif`,n0=`#ifdef USE_EMISSIVEMAP
	vec4 emissiveColor = texture2D( emissiveMap, vEmissiveMapUv );
	#ifdef DECODE_VIDEO_TEXTURE_EMISSIVE
		emissiveColor = sRGBTransferEOTF( emissiveColor );
	#endif
	totalEmissiveRadiance *= emissiveColor.rgb;
#endif`,i0=`#ifdef USE_EMISSIVEMAP
	uniform sampler2D emissiveMap;
#endif`,s0="gl_FragColor = linearToOutputTexel( gl_FragColor );",r0=`vec4 LinearTransferOETF( in vec4 value ) {
	return value;
}
vec4 sRGBTransferEOTF( in vec4 value ) {
	return vec4( mix( pow( value.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), value.rgb * 0.0773993808, vec3( lessThanEqual( value.rgb, vec3( 0.04045 ) ) ) ), value.a );
}
vec4 sRGBTransferOETF( in vec4 value ) {
	return vec4( mix( pow( value.rgb, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 ), value.rgb * 12.92, vec3( lessThanEqual( value.rgb, vec3( 0.0031308 ) ) ) ), value.a );
}`,a0=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vec3 cameraToFrag;
		if ( isOrthographic ) {
			cameraToFrag = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToFrag = normalize( vWorldPosition - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vec3 reflectVec = reflect( cameraToFrag, worldNormal );
		#else
			vec3 reflectVec = refract( cameraToFrag, worldNormal, refractionRatio );
		#endif
	#else
		vec3 reflectVec = vReflect;
	#endif
	#ifdef ENVMAP_TYPE_CUBE
		vec4 envColor = textureCube( envMap, envMapRotation * vec3( flipEnvMap * reflectVec.x, reflectVec.yz ) );
	#else
		vec4 envColor = vec4( 0.0 );
	#endif
	#ifdef ENVMAP_BLENDING_MULTIPLY
		outgoingLight = mix( outgoingLight, outgoingLight * envColor.xyz, specularStrength * reflectivity );
	#elif defined( ENVMAP_BLENDING_MIX )
		outgoingLight = mix( outgoingLight, envColor.xyz, specularStrength * reflectivity );
	#elif defined( ENVMAP_BLENDING_ADD )
		outgoingLight += envColor.xyz * specularStrength * reflectivity;
	#endif
#endif`,o0=`#ifdef USE_ENVMAP
	uniform float envMapIntensity;
	uniform float flipEnvMap;
	uniform mat3 envMapRotation;
	#ifdef ENVMAP_TYPE_CUBE
		uniform samplerCube envMap;
	#else
		uniform sampler2D envMap;
	#endif
	
#endif`,l0=`#ifdef USE_ENVMAP
	uniform float reflectivity;
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		varying vec3 vWorldPosition;
		uniform float refractionRatio;
	#else
		varying vec3 vReflect;
	#endif
#endif`,c0=`#ifdef USE_ENVMAP
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		
		varying vec3 vWorldPosition;
	#else
		varying vec3 vReflect;
		uniform float refractionRatio;
	#endif
#endif`,h0=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vWorldPosition = worldPosition.xyz;
	#else
		vec3 cameraToVertex;
		if ( isOrthographic ) {
			cameraToVertex = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToVertex = normalize( worldPosition.xyz - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vReflect = reflect( cameraToVertex, worldNormal );
		#else
			vReflect = refract( cameraToVertex, worldNormal, refractionRatio );
		#endif
	#endif
#endif`,u0=`#ifdef USE_FOG
	vFogDepth = - mvPosition.z;
#endif`,d0=`#ifdef USE_FOG
	varying float vFogDepth;
#endif`,f0=`#ifdef USE_FOG
	#ifdef FOG_EXP2
		float fogFactor = 1.0 - exp( - fogDensity * fogDensity * vFogDepth * vFogDepth );
	#else
		float fogFactor = smoothstep( fogNear, fogFar, vFogDepth );
	#endif
	gl_FragColor.rgb = mix( gl_FragColor.rgb, fogColor, fogFactor );
#endif`,p0=`#ifdef USE_FOG
	uniform vec3 fogColor;
	varying float vFogDepth;
	#ifdef FOG_EXP2
		uniform float fogDensity;
	#else
		uniform float fogNear;
		uniform float fogFar;
	#endif
#endif`,m0=`#ifdef USE_GRADIENTMAP
	uniform sampler2D gradientMap;
#endif
vec3 getGradientIrradiance( vec3 normal, vec3 lightDirection ) {
	float dotNL = dot( normal, lightDirection );
	vec2 coord = vec2( dotNL * 0.5 + 0.5, 0.0 );
	#ifdef USE_GRADIENTMAP
		return vec3( texture2D( gradientMap, coord ).r );
	#else
		vec2 fw = fwidth( coord ) * 0.5;
		return mix( vec3( 0.7 ), vec3( 1.0 ), smoothstep( 0.7 - fw.x, 0.7 + fw.x, coord.x ) );
	#endif
}`,g0=`#ifdef USE_LIGHTMAP
	uniform sampler2D lightMap;
	uniform float lightMapIntensity;
#endif`,x0=`LambertMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularStrength = specularStrength;`,v0=`varying vec3 vViewPosition;
struct LambertMaterial {
	vec3 diffuseColor;
	float specularStrength;
};
void RE_Direct_Lambert( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Lambert( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Lambert
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Lambert`,y0=`uniform bool receiveShadow;
uniform vec3 ambientLightColor;
#if defined( USE_LIGHT_PROBES )
	uniform vec3 lightProbe[ 9 ];
#endif
vec3 shGetIrradianceAt( in vec3 normal, in vec3 shCoefficients[ 9 ] ) {
	float x = normal.x, y = normal.y, z = normal.z;
	vec3 result = shCoefficients[ 0 ] * 0.886227;
	result += shCoefficients[ 1 ] * 2.0 * 0.511664 * y;
	result += shCoefficients[ 2 ] * 2.0 * 0.511664 * z;
	result += shCoefficients[ 3 ] * 2.0 * 0.511664 * x;
	result += shCoefficients[ 4 ] * 2.0 * 0.429043 * x * y;
	result += shCoefficients[ 5 ] * 2.0 * 0.429043 * y * z;
	result += shCoefficients[ 6 ] * ( 0.743125 * z * z - 0.247708 );
	result += shCoefficients[ 7 ] * 2.0 * 0.429043 * x * z;
	result += shCoefficients[ 8 ] * 0.429043 * ( x * x - y * y );
	return result;
}
vec3 getLightProbeIrradiance( const in vec3 lightProbe[ 9 ], const in vec3 normal ) {
	vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
	vec3 irradiance = shGetIrradianceAt( worldNormal, lightProbe );
	return irradiance;
}
vec3 getAmbientLightIrradiance( const in vec3 ambientLightColor ) {
	vec3 irradiance = ambientLightColor;
	return irradiance;
}
float getDistanceAttenuation( const in float lightDistance, const in float cutoffDistance, const in float decayExponent ) {
	float distanceFalloff = 1.0 / max( pow( lightDistance, decayExponent ), 0.01 );
	if ( cutoffDistance > 0.0 ) {
		distanceFalloff *= pow2( saturate( 1.0 - pow4( lightDistance / cutoffDistance ) ) );
	}
	return distanceFalloff;
}
float getSpotAttenuation( const in float coneCosine, const in float penumbraCosine, const in float angleCosine ) {
	return smoothstep( coneCosine, penumbraCosine, angleCosine );
}
#if NUM_DIR_LIGHTS > 0
	struct DirectionalLight {
		vec3 direction;
		vec3 color;
	};
	uniform DirectionalLight directionalLights[ NUM_DIR_LIGHTS ];
	void getDirectionalLightInfo( const in DirectionalLight directionalLight, out IncidentLight light ) {
		light.color = directionalLight.color;
		light.direction = directionalLight.direction;
		light.visible = true;
	}
#endif
#if NUM_POINT_LIGHTS > 0
	struct PointLight {
		vec3 position;
		vec3 color;
		float distance;
		float decay;
	};
	uniform PointLight pointLights[ NUM_POINT_LIGHTS ];
	void getPointLightInfo( const in PointLight pointLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = pointLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float lightDistance = length( lVector );
		light.color = pointLight.color;
		light.color *= getDistanceAttenuation( lightDistance, pointLight.distance, pointLight.decay );
		light.visible = ( light.color != vec3( 0.0 ) );
	}
#endif
#if NUM_SPOT_LIGHTS > 0
	struct SpotLight {
		vec3 position;
		vec3 direction;
		vec3 color;
		float distance;
		float decay;
		float coneCos;
		float penumbraCos;
	};
	uniform SpotLight spotLights[ NUM_SPOT_LIGHTS ];
	void getSpotLightInfo( const in SpotLight spotLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = spotLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float angleCos = dot( light.direction, spotLight.direction );
		float spotAttenuation = getSpotAttenuation( spotLight.coneCos, spotLight.penumbraCos, angleCos );
		if ( spotAttenuation > 0.0 ) {
			float lightDistance = length( lVector );
			light.color = spotLight.color * spotAttenuation;
			light.color *= getDistanceAttenuation( lightDistance, spotLight.distance, spotLight.decay );
			light.visible = ( light.color != vec3( 0.0 ) );
		} else {
			light.color = vec3( 0.0 );
			light.visible = false;
		}
	}
#endif
#if NUM_RECT_AREA_LIGHTS > 0
	struct RectAreaLight {
		vec3 color;
		vec3 position;
		vec3 halfWidth;
		vec3 halfHeight;
	};
	uniform sampler2D ltc_1;	uniform sampler2D ltc_2;
	uniform RectAreaLight rectAreaLights[ NUM_RECT_AREA_LIGHTS ];
#endif
#if NUM_HEMI_LIGHTS > 0
	struct HemisphereLight {
		vec3 direction;
		vec3 skyColor;
		vec3 groundColor;
	};
	uniform HemisphereLight hemisphereLights[ NUM_HEMI_LIGHTS ];
	vec3 getHemisphereLightIrradiance( const in HemisphereLight hemiLight, const in vec3 normal ) {
		float dotNL = dot( normal, hemiLight.direction );
		float hemiDiffuseWeight = 0.5 * dotNL + 0.5;
		vec3 irradiance = mix( hemiLight.groundColor, hemiLight.skyColor, hemiDiffuseWeight );
		return irradiance;
	}
#endif`,_0=`#ifdef USE_ENVMAP
	vec3 getIBLIrradiance( const in vec3 normal ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * worldNormal, 1.0 );
			return PI * envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	vec3 getIBLRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 reflectVec = reflect( - viewDir, normal );
			reflectVec = normalize( mix( reflectVec, normal, roughness * roughness) );
			reflectVec = inverseTransformDirection( reflectVec, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * reflectVec, roughness );
			return envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	#ifdef USE_ANISOTROPY
		vec3 getIBLAnisotropyRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness, const in vec3 bitangent, const in float anisotropy ) {
			#ifdef ENVMAP_TYPE_CUBE_UV
				vec3 bentNormal = cross( bitangent, viewDir );
				bentNormal = normalize( cross( bentNormal, bitangent ) );
				bentNormal = normalize( mix( bentNormal, normal, pow2( pow2( 1.0 - anisotropy * ( 1.0 - roughness ) ) ) ) );
				return getIBLRadiance( viewDir, bentNormal, roughness );
			#else
				return vec3( 0.0 );
			#endif
		}
	#endif
#endif`,b0=`ToonMaterial material;
material.diffuseColor = diffuseColor.rgb;`,M0=`varying vec3 vViewPosition;
struct ToonMaterial {
	vec3 diffuseColor;
};
void RE_Direct_Toon( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	vec3 irradiance = getGradientIrradiance( geometryNormal, directLight.direction ) * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Toon( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Toon
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Toon`,S0=`BlinnPhongMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularColor = specular;
material.specularShininess = shininess;
material.specularStrength = specularStrength;`,w0=`varying vec3 vViewPosition;
struct BlinnPhongMaterial {
	vec3 diffuseColor;
	vec3 specularColor;
	float specularShininess;
	float specularStrength;
};
void RE_Direct_BlinnPhong( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
	reflectedLight.directSpecular += irradiance * BRDF_BlinnPhong( directLight.direction, geometryViewDir, geometryNormal, material.specularColor, material.specularShininess ) * material.specularStrength;
}
void RE_IndirectDiffuse_BlinnPhong( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_BlinnPhong
#define RE_IndirectDiffuse		RE_IndirectDiffuse_BlinnPhong`,E0=`PhysicalMaterial material;
material.diffuseColor = diffuseColor.rgb * ( 1.0 - metalnessFactor );
vec3 dxy = max( abs( dFdx( nonPerturbedNormal ) ), abs( dFdy( nonPerturbedNormal ) ) );
float geometryRoughness = max( max( dxy.x, dxy.y ), dxy.z );
material.roughness = max( roughnessFactor, 0.0525 );material.roughness += geometryRoughness;
material.roughness = min( material.roughness, 1.0 );
#ifdef IOR
	material.ior = ior;
	#ifdef USE_SPECULAR
		float specularIntensityFactor = specularIntensity;
		vec3 specularColorFactor = specularColor;
		#ifdef USE_SPECULAR_COLORMAP
			specularColorFactor *= texture2D( specularColorMap, vSpecularColorMapUv ).rgb;
		#endif
		#ifdef USE_SPECULAR_INTENSITYMAP
			specularIntensityFactor *= texture2D( specularIntensityMap, vSpecularIntensityMapUv ).a;
		#endif
		material.specularF90 = mix( specularIntensityFactor, 1.0, metalnessFactor );
	#else
		float specularIntensityFactor = 1.0;
		vec3 specularColorFactor = vec3( 1.0 );
		material.specularF90 = 1.0;
	#endif
	material.specularColor = mix( min( pow2( ( material.ior - 1.0 ) / ( material.ior + 1.0 ) ) * specularColorFactor, vec3( 1.0 ) ) * specularIntensityFactor, diffuseColor.rgb, metalnessFactor );
#else
	material.specularColor = mix( vec3( 0.04 ), diffuseColor.rgb, metalnessFactor );
	material.specularF90 = 1.0;
#endif
#ifdef USE_CLEARCOAT
	material.clearcoat = clearcoat;
	material.clearcoatRoughness = clearcoatRoughness;
	material.clearcoatF0 = vec3( 0.04 );
	material.clearcoatF90 = 1.0;
	#ifdef USE_CLEARCOATMAP
		material.clearcoat *= texture2D( clearcoatMap, vClearcoatMapUv ).x;
	#endif
	#ifdef USE_CLEARCOAT_ROUGHNESSMAP
		material.clearcoatRoughness *= texture2D( clearcoatRoughnessMap, vClearcoatRoughnessMapUv ).y;
	#endif
	material.clearcoat = saturate( material.clearcoat );	material.clearcoatRoughness = max( material.clearcoatRoughness, 0.0525 );
	material.clearcoatRoughness += geometryRoughness;
	material.clearcoatRoughness = min( material.clearcoatRoughness, 1.0 );
#endif
#ifdef USE_DISPERSION
	material.dispersion = dispersion;
#endif
#ifdef USE_IRIDESCENCE
	material.iridescence = iridescence;
	material.iridescenceIOR = iridescenceIOR;
	#ifdef USE_IRIDESCENCEMAP
		material.iridescence *= texture2D( iridescenceMap, vIridescenceMapUv ).r;
	#endif
	#ifdef USE_IRIDESCENCE_THICKNESSMAP
		material.iridescenceThickness = (iridescenceThicknessMaximum - iridescenceThicknessMinimum) * texture2D( iridescenceThicknessMap, vIridescenceThicknessMapUv ).g + iridescenceThicknessMinimum;
	#else
		material.iridescenceThickness = iridescenceThicknessMaximum;
	#endif
#endif
#ifdef USE_SHEEN
	material.sheenColor = sheenColor;
	#ifdef USE_SHEEN_COLORMAP
		material.sheenColor *= texture2D( sheenColorMap, vSheenColorMapUv ).rgb;
	#endif
	material.sheenRoughness = clamp( sheenRoughness, 0.07, 1.0 );
	#ifdef USE_SHEEN_ROUGHNESSMAP
		material.sheenRoughness *= texture2D( sheenRoughnessMap, vSheenRoughnessMapUv ).a;
	#endif
#endif
#ifdef USE_ANISOTROPY
	#ifdef USE_ANISOTROPYMAP
		mat2 anisotropyMat = mat2( anisotropyVector.x, anisotropyVector.y, - anisotropyVector.y, anisotropyVector.x );
		vec3 anisotropyPolar = texture2D( anisotropyMap, vAnisotropyMapUv ).rgb;
		vec2 anisotropyV = anisotropyMat * normalize( 2.0 * anisotropyPolar.rg - vec2( 1.0 ) ) * anisotropyPolar.b;
	#else
		vec2 anisotropyV = anisotropyVector;
	#endif
	material.anisotropy = length( anisotropyV );
	if( material.anisotropy == 0.0 ) {
		anisotropyV = vec2( 1.0, 0.0 );
	} else {
		anisotropyV /= material.anisotropy;
		material.anisotropy = saturate( material.anisotropy );
	}
	material.alphaT = mix( pow2( material.roughness ), 1.0, pow2( material.anisotropy ) );
	material.anisotropyT = tbn[ 0 ] * anisotropyV.x + tbn[ 1 ] * anisotropyV.y;
	material.anisotropyB = tbn[ 1 ] * anisotropyV.x - tbn[ 0 ] * anisotropyV.y;
#endif`,T0=`struct PhysicalMaterial {
	vec3 diffuseColor;
	float roughness;
	vec3 specularColor;
	float specularF90;
	float dispersion;
	#ifdef USE_CLEARCOAT
		float clearcoat;
		float clearcoatRoughness;
		vec3 clearcoatF0;
		float clearcoatF90;
	#endif
	#ifdef USE_IRIDESCENCE
		float iridescence;
		float iridescenceIOR;
		float iridescenceThickness;
		vec3 iridescenceFresnel;
		vec3 iridescenceF0;
	#endif
	#ifdef USE_SHEEN
		vec3 sheenColor;
		float sheenRoughness;
	#endif
	#ifdef IOR
		float ior;
	#endif
	#ifdef USE_TRANSMISSION
		float transmission;
		float transmissionAlpha;
		float thickness;
		float attenuationDistance;
		vec3 attenuationColor;
	#endif
	#ifdef USE_ANISOTROPY
		float anisotropy;
		float alphaT;
		vec3 anisotropyT;
		vec3 anisotropyB;
	#endif
};
vec3 clearcoatSpecularDirect = vec3( 0.0 );
vec3 clearcoatSpecularIndirect = vec3( 0.0 );
vec3 sheenSpecularDirect = vec3( 0.0 );
vec3 sheenSpecularIndirect = vec3(0.0 );
vec3 Schlick_to_F0( const in vec3 f, const in float f90, const in float dotVH ) {
    float x = clamp( 1.0 - dotVH, 0.0, 1.0 );
    float x2 = x * x;
    float x5 = clamp( x * x2 * x2, 0.0, 0.9999 );
    return ( f - vec3( f90 ) * x5 ) / ( 1.0 - x5 );
}
float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV ) {
	float a2 = pow2( alpha );
	float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
	float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
	return 0.5 / max( gv + gl, EPSILON );
}
float D_GGX( const in float alpha, const in float dotNH ) {
	float a2 = pow2( alpha );
	float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
	return RECIPROCAL_PI * a2 / pow2( denom );
}
#ifdef USE_ANISOTROPY
	float V_GGX_SmithCorrelated_Anisotropic( const in float alphaT, const in float alphaB, const in float dotTV, const in float dotBV, const in float dotTL, const in float dotBL, const in float dotNV, const in float dotNL ) {
		float gv = dotNL * length( vec3( alphaT * dotTV, alphaB * dotBV, dotNV ) );
		float gl = dotNV * length( vec3( alphaT * dotTL, alphaB * dotBL, dotNL ) );
		float v = 0.5 / ( gv + gl );
		return saturate(v);
	}
	float D_GGX_Anisotropic( const in float alphaT, const in float alphaB, const in float dotNH, const in float dotTH, const in float dotBH ) {
		float a2 = alphaT * alphaB;
		highp vec3 v = vec3( alphaB * dotTH, alphaT * dotBH, a2 * dotNH );
		highp float v2 = dot( v, v );
		float w2 = a2 / v2;
		return RECIPROCAL_PI * a2 * pow2 ( w2 );
	}
#endif
#ifdef USE_CLEARCOAT
	vec3 BRDF_GGX_Clearcoat( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material) {
		vec3 f0 = material.clearcoatF0;
		float f90 = material.clearcoatF90;
		float roughness = material.clearcoatRoughness;
		float alpha = pow2( roughness );
		vec3 halfDir = normalize( lightDir + viewDir );
		float dotNL = saturate( dot( normal, lightDir ) );
		float dotNV = saturate( dot( normal, viewDir ) );
		float dotNH = saturate( dot( normal, halfDir ) );
		float dotVH = saturate( dot( viewDir, halfDir ) );
		vec3 F = F_Schlick( f0, f90, dotVH );
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
		return F * ( V * D );
	}
#endif
vec3 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) {
	vec3 f0 = material.specularColor;
	float f90 = material.specularF90;
	float roughness = material.roughness;
	float alpha = pow2( roughness );
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( f0, f90, dotVH );
	#ifdef USE_IRIDESCENCE
		F = mix( F, material.iridescenceFresnel, material.iridescence );
	#endif
	#ifdef USE_ANISOTROPY
		float dotTL = dot( material.anisotropyT, lightDir );
		float dotTV = dot( material.anisotropyT, viewDir );
		float dotTH = dot( material.anisotropyT, halfDir );
		float dotBL = dot( material.anisotropyB, lightDir );
		float dotBV = dot( material.anisotropyB, viewDir );
		float dotBH = dot( material.anisotropyB, halfDir );
		float V = V_GGX_SmithCorrelated_Anisotropic( material.alphaT, alpha, dotTV, dotBV, dotTL, dotBL, dotNV, dotNL );
		float D = D_GGX_Anisotropic( material.alphaT, alpha, dotNH, dotTH, dotBH );
	#else
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
	#endif
	return F * ( V * D );
}
vec2 LTC_Uv( const in vec3 N, const in vec3 V, const in float roughness ) {
	const float LUT_SIZE = 64.0;
	const float LUT_SCALE = ( LUT_SIZE - 1.0 ) / LUT_SIZE;
	const float LUT_BIAS = 0.5 / LUT_SIZE;
	float dotNV = saturate( dot( N, V ) );
	vec2 uv = vec2( roughness, sqrt( 1.0 - dotNV ) );
	uv = uv * LUT_SCALE + LUT_BIAS;
	return uv;
}
float LTC_ClippedSphereFormFactor( const in vec3 f ) {
	float l = length( f );
	return max( ( l * l + f.z ) / ( l + 1.0 ), 0.0 );
}
vec3 LTC_EdgeVectorFormFactor( const in vec3 v1, const in vec3 v2 ) {
	float x = dot( v1, v2 );
	float y = abs( x );
	float a = 0.8543985 + ( 0.4965155 + 0.0145206 * y ) * y;
	float b = 3.4175940 + ( 4.1616724 + y ) * y;
	float v = a / b;
	float theta_sintheta = ( x > 0.0 ) ? v : 0.5 * inversesqrt( max( 1.0 - x * x, 1e-7 ) ) - v;
	return cross( v1, v2 ) * theta_sintheta;
}
vec3 LTC_Evaluate( const in vec3 N, const in vec3 V, const in vec3 P, const in mat3 mInv, const in vec3 rectCoords[ 4 ] ) {
	vec3 v1 = rectCoords[ 1 ] - rectCoords[ 0 ];
	vec3 v2 = rectCoords[ 3 ] - rectCoords[ 0 ];
	vec3 lightNormal = cross( v1, v2 );
	if( dot( lightNormal, P - rectCoords[ 0 ] ) < 0.0 ) return vec3( 0.0 );
	vec3 T1, T2;
	T1 = normalize( V - N * dot( V, N ) );
	T2 = - cross( N, T1 );
	mat3 mat = mInv * transposeMat3( mat3( T1, T2, N ) );
	vec3 coords[ 4 ];
	coords[ 0 ] = mat * ( rectCoords[ 0 ] - P );
	coords[ 1 ] = mat * ( rectCoords[ 1 ] - P );
	coords[ 2 ] = mat * ( rectCoords[ 2 ] - P );
	coords[ 3 ] = mat * ( rectCoords[ 3 ] - P );
	coords[ 0 ] = normalize( coords[ 0 ] );
	coords[ 1 ] = normalize( coords[ 1 ] );
	coords[ 2 ] = normalize( coords[ 2 ] );
	coords[ 3 ] = normalize( coords[ 3 ] );
	vec3 vectorFormFactor = vec3( 0.0 );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 0 ], coords[ 1 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 1 ], coords[ 2 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 2 ], coords[ 3 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 3 ], coords[ 0 ] );
	float result = LTC_ClippedSphereFormFactor( vectorFormFactor );
	return vec3( result );
}
#if defined( USE_SHEEN )
float D_Charlie( float roughness, float dotNH ) {
	float alpha = pow2( roughness );
	float invAlpha = 1.0 / alpha;
	float cos2h = dotNH * dotNH;
	float sin2h = max( 1.0 - cos2h, 0.0078125 );
	return ( 2.0 + invAlpha ) * pow( sin2h, invAlpha * 0.5 ) / ( 2.0 * PI );
}
float V_Neubelt( float dotNV, float dotNL ) {
	return saturate( 1.0 / ( 4.0 * ( dotNL + dotNV - dotNL * dotNV ) ) );
}
vec3 BRDF_Sheen( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, vec3 sheenColor, const in float sheenRoughness ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float D = D_Charlie( sheenRoughness, dotNH );
	float V = V_Neubelt( dotNV, dotNL );
	return sheenColor * ( D * V );
}
#endif
float IBLSheenBRDF( const in vec3 normal, const in vec3 viewDir, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	float r2 = roughness * roughness;
	float a = roughness < 0.25 ? -339.2 * r2 + 161.4 * roughness - 25.9 : -8.48 * r2 + 14.3 * roughness - 9.95;
	float b = roughness < 0.25 ? 44.0 * r2 - 23.7 * roughness + 3.26 : 1.97 * r2 - 3.27 * roughness + 0.72;
	float DG = exp( a * dotNV + b ) + ( roughness < 0.25 ? 0.0 : 0.1 * ( roughness - 0.25 ) );
	return saturate( DG * RECIPROCAL_PI );
}
vec2 DFGApprox( const in vec3 normal, const in vec3 viewDir, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	const vec4 c0 = vec4( - 1, - 0.0275, - 0.572, 0.022 );
	const vec4 c1 = vec4( 1, 0.0425, 1.04, - 0.04 );
	vec4 r = roughness * c0 + c1;
	float a004 = min( r.x * r.x, exp2( - 9.28 * dotNV ) ) * r.x + r.y;
	vec2 fab = vec2( - 1.04, 1.04 ) * a004 + r.zw;
	return fab;
}
vec3 EnvironmentBRDF( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness ) {
	vec2 fab = DFGApprox( normal, viewDir, roughness );
	return specularColor * fab.x + specularF90 * fab.y;
}
#ifdef USE_IRIDESCENCE
void computeMultiscatteringIridescence( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float iridescence, const in vec3 iridescenceF0, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#else
void computeMultiscattering( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#endif
	vec2 fab = DFGApprox( normal, viewDir, roughness );
	#ifdef USE_IRIDESCENCE
		vec3 Fr = mix( specularColor, iridescenceF0, iridescence );
	#else
		vec3 Fr = specularColor;
	#endif
	vec3 FssEss = Fr * fab.x + specularF90 * fab.y;
	float Ess = fab.x + fab.y;
	float Ems = 1.0 - Ess;
	vec3 Favg = Fr + ( 1.0 - Fr ) * 0.047619;	vec3 Fms = FssEss * Favg / ( 1.0 - Ems * Favg );
	singleScatter += FssEss;
	multiScatter += Fms * Ems;
}
#if NUM_RECT_AREA_LIGHTS > 0
	void RE_Direct_RectArea_Physical( const in RectAreaLight rectAreaLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
		vec3 normal = geometryNormal;
		vec3 viewDir = geometryViewDir;
		vec3 position = geometryPosition;
		vec3 lightPos = rectAreaLight.position;
		vec3 halfWidth = rectAreaLight.halfWidth;
		vec3 halfHeight = rectAreaLight.halfHeight;
		vec3 lightColor = rectAreaLight.color;
		float roughness = material.roughness;
		vec3 rectCoords[ 4 ];
		rectCoords[ 0 ] = lightPos + halfWidth - halfHeight;		rectCoords[ 1 ] = lightPos - halfWidth - halfHeight;
		rectCoords[ 2 ] = lightPos - halfWidth + halfHeight;
		rectCoords[ 3 ] = lightPos + halfWidth + halfHeight;
		vec2 uv = LTC_Uv( normal, viewDir, roughness );
		vec4 t1 = texture2D( ltc_1, uv );
		vec4 t2 = texture2D( ltc_2, uv );
		mat3 mInv = mat3(
			vec3( t1.x, 0, t1.y ),
			vec3(    0, 1,    0 ),
			vec3( t1.z, 0, t1.w )
		);
		vec3 fresnel = ( material.specularColor * t2.x + ( vec3( 1.0 ) - material.specularColor ) * t2.y );
		reflectedLight.directSpecular += lightColor * fresnel * LTC_Evaluate( normal, viewDir, position, mInv, rectCoords );
		reflectedLight.directDiffuse += lightColor * material.diffuseColor * LTC_Evaluate( normal, viewDir, position, mat3( 1.0 ), rectCoords );
	}
#endif
void RE_Direct_Physical( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	#ifdef USE_CLEARCOAT
		float dotNLcc = saturate( dot( geometryClearcoatNormal, directLight.direction ) );
		vec3 ccIrradiance = dotNLcc * directLight.color;
		clearcoatSpecularDirect += ccIrradiance * BRDF_GGX_Clearcoat( directLight.direction, geometryViewDir, geometryClearcoatNormal, material );
	#endif
	#ifdef USE_SHEEN
		sheenSpecularDirect += irradiance * BRDF_Sheen( directLight.direction, geometryViewDir, geometryNormal, material.sheenColor, material.sheenRoughness );
	#endif
	reflectedLight.directSpecular += irradiance * BRDF_GGX( directLight.direction, geometryViewDir, geometryNormal, material );
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Physical( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectSpecular_Physical( const in vec3 radiance, const in vec3 irradiance, const in vec3 clearcoatRadiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight) {
	#ifdef USE_CLEARCOAT
		clearcoatSpecularIndirect += clearcoatRadiance * EnvironmentBRDF( geometryClearcoatNormal, geometryViewDir, material.clearcoatF0, material.clearcoatF90, material.clearcoatRoughness );
	#endif
	#ifdef USE_SHEEN
		sheenSpecularIndirect += irradiance * material.sheenColor * IBLSheenBRDF( geometryNormal, geometryViewDir, material.sheenRoughness );
	#endif
	vec3 singleScattering = vec3( 0.0 );
	vec3 multiScattering = vec3( 0.0 );
	vec3 cosineWeightedIrradiance = irradiance * RECIPROCAL_PI;
	#ifdef USE_IRIDESCENCE
		computeMultiscatteringIridescence( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.iridescence, material.iridescenceFresnel, material.roughness, singleScattering, multiScattering );
	#else
		computeMultiscattering( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.roughness, singleScattering, multiScattering );
	#endif
	vec3 totalScattering = singleScattering + multiScattering;
	vec3 diffuse = material.diffuseColor * ( 1.0 - max( max( totalScattering.r, totalScattering.g ), totalScattering.b ) );
	reflectedLight.indirectSpecular += radiance * singleScattering;
	reflectedLight.indirectSpecular += multiScattering * cosineWeightedIrradiance;
	reflectedLight.indirectDiffuse += diffuse * cosineWeightedIrradiance;
}
#define RE_Direct				RE_Direct_Physical
#define RE_Direct_RectArea		RE_Direct_RectArea_Physical
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Physical
#define RE_IndirectSpecular		RE_IndirectSpecular_Physical
float computeSpecularOcclusion( const in float dotNV, const in float ambientOcclusion, const in float roughness ) {
	return saturate( pow( dotNV + ambientOcclusion, exp2( - 16.0 * roughness - 1.0 ) ) - 1.0 + ambientOcclusion );
}`,A0=`
vec3 geometryPosition = - vViewPosition;
vec3 geometryNormal = normal;
vec3 geometryViewDir = ( isOrthographic ) ? vec3( 0, 0, 1 ) : normalize( vViewPosition );
vec3 geometryClearcoatNormal = vec3( 0.0 );
#ifdef USE_CLEARCOAT
	geometryClearcoatNormal = clearcoatNormal;
#endif
#ifdef USE_IRIDESCENCE
	float dotNVi = saturate( dot( normal, geometryViewDir ) );
	if ( material.iridescenceThickness == 0.0 ) {
		material.iridescence = 0.0;
	} else {
		material.iridescence = saturate( material.iridescence );
	}
	if ( material.iridescence > 0.0 ) {
		material.iridescenceFresnel = evalIridescence( 1.0, material.iridescenceIOR, dotNVi, material.iridescenceThickness, material.specularColor );
		material.iridescenceF0 = Schlick_to_F0( material.iridescenceFresnel, 1.0, dotNVi );
	}
#endif
IncidentLight directLight;
#if ( NUM_POINT_LIGHTS > 0 ) && defined( RE_Direct )
	PointLight pointLight;
	#if defined( USE_SHADOWMAP ) && NUM_POINT_LIGHT_SHADOWS > 0
	PointLightShadow pointLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHTS; i ++ ) {
		pointLight = pointLights[ i ];
		getPointLightInfo( pointLight, geometryPosition, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_POINT_LIGHT_SHADOWS )
		pointLightShadow = pointLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getPointShadow( pointShadowMap[ i ], pointLightShadow.shadowMapSize, pointLightShadow.shadowIntensity, pointLightShadow.shadowBias, pointLightShadow.shadowRadius, vPointShadowCoord[ i ], pointLightShadow.shadowCameraNear, pointLightShadow.shadowCameraFar ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_SPOT_LIGHTS > 0 ) && defined( RE_Direct )
	SpotLight spotLight;
	vec4 spotColor;
	vec3 spotLightCoord;
	bool inSpotLightMap;
	#if defined( USE_SHADOWMAP ) && NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHTS; i ++ ) {
		spotLight = spotLights[ i ];
		getSpotLightInfo( spotLight, geometryPosition, directLight );
		#if ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#define SPOT_LIGHT_MAP_INDEX UNROLLED_LOOP_INDEX
		#elif ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		#define SPOT_LIGHT_MAP_INDEX NUM_SPOT_LIGHT_MAPS
		#else
		#define SPOT_LIGHT_MAP_INDEX ( UNROLLED_LOOP_INDEX - NUM_SPOT_LIGHT_SHADOWS + NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#endif
		#if ( SPOT_LIGHT_MAP_INDEX < NUM_SPOT_LIGHT_MAPS )
			spotLightCoord = vSpotLightCoord[ i ].xyz / vSpotLightCoord[ i ].w;
			inSpotLightMap = all( lessThan( abs( spotLightCoord * 2. - 1. ), vec3( 1.0 ) ) );
			spotColor = texture2D( spotLightMap[ SPOT_LIGHT_MAP_INDEX ], spotLightCoord.xy );
			directLight.color = inSpotLightMap ? directLight.color * spotColor.rgb : directLight.color;
		#endif
		#undef SPOT_LIGHT_MAP_INDEX
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		spotLightShadow = spotLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( spotShadowMap[ i ], spotLightShadow.shadowMapSize, spotLightShadow.shadowIntensity, spotLightShadow.shadowBias, spotLightShadow.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_DIR_LIGHTS > 0 ) && defined( RE_Direct )
	DirectionalLight directionalLight;
	#if defined( USE_SHADOWMAP ) && NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHTS; i ++ ) {
		directionalLight = directionalLights[ i ];
		getDirectionalLightInfo( directionalLight, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_DIR_LIGHT_SHADOWS )
		directionalLightShadow = directionalLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( directionalShadowMap[ i ], directionalLightShadow.shadowMapSize, directionalLightShadow.shadowIntensity, directionalLightShadow.shadowBias, directionalLightShadow.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_RECT_AREA_LIGHTS > 0 ) && defined( RE_Direct_RectArea )
	RectAreaLight rectAreaLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_RECT_AREA_LIGHTS; i ++ ) {
		rectAreaLight = rectAreaLights[ i ];
		RE_Direct_RectArea( rectAreaLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if defined( RE_IndirectDiffuse )
	vec3 iblIrradiance = vec3( 0.0 );
	vec3 irradiance = getAmbientLightIrradiance( ambientLightColor );
	#if defined( USE_LIGHT_PROBES )
		irradiance += getLightProbeIrradiance( lightProbe, geometryNormal );
	#endif
	#if ( NUM_HEMI_LIGHTS > 0 )
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_HEMI_LIGHTS; i ++ ) {
			irradiance += getHemisphereLightIrradiance( hemisphereLights[ i ], geometryNormal );
		}
		#pragma unroll_loop_end
	#endif
#endif
#if defined( RE_IndirectSpecular )
	vec3 radiance = vec3( 0.0 );
	vec3 clearcoatRadiance = vec3( 0.0 );
#endif`,C0=`#if defined( RE_IndirectDiffuse )
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		vec3 lightMapIrradiance = lightMapTexel.rgb * lightMapIntensity;
		irradiance += lightMapIrradiance;
	#endif
	#if defined( USE_ENVMAP ) && defined( STANDARD ) && defined( ENVMAP_TYPE_CUBE_UV )
		iblIrradiance += getIBLIrradiance( geometryNormal );
	#endif
#endif
#if defined( USE_ENVMAP ) && defined( RE_IndirectSpecular )
	#ifdef USE_ANISOTROPY
		radiance += getIBLAnisotropyRadiance( geometryViewDir, geometryNormal, material.roughness, material.anisotropyB, material.anisotropy );
	#else
		radiance += getIBLRadiance( geometryViewDir, geometryNormal, material.roughness );
	#endif
	#ifdef USE_CLEARCOAT
		clearcoatRadiance += getIBLRadiance( geometryViewDir, geometryClearcoatNormal, material.clearcoatRoughness );
	#endif
#endif`,R0=`#if defined( RE_IndirectDiffuse )
	RE_IndirectDiffuse( irradiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif
#if defined( RE_IndirectSpecular )
	RE_IndirectSpecular( radiance, iblIrradiance, clearcoatRadiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif`,P0=`#if defined( USE_LOGDEPTHBUF )
	gl_FragDepth = vIsPerspective == 0.0 ? gl_FragCoord.z : log2( vFragDepth ) * logDepthBufFC * 0.5;
#endif`,I0=`#if defined( USE_LOGDEPTHBUF )
	uniform float logDepthBufFC;
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,F0=`#ifdef USE_LOGDEPTHBUF
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,D0=`#ifdef USE_LOGDEPTHBUF
	vFragDepth = 1.0 + gl_Position.w;
	vIsPerspective = float( isPerspectiveMatrix( projectionMatrix ) );
#endif`,k0=`#ifdef USE_MAP
	vec4 sampledDiffuseColor = texture2D( map, vMapUv );
	#ifdef DECODE_VIDEO_TEXTURE
		sampledDiffuseColor = sRGBTransferEOTF( sampledDiffuseColor );
	#endif
	diffuseColor *= sampledDiffuseColor;
#endif`,L0=`#ifdef USE_MAP
	uniform sampler2D map;
#endif`,U0=`#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
	#if defined( USE_POINTS_UV )
		vec2 uv = vUv;
	#else
		vec2 uv = ( uvTransform * vec3( gl_PointCoord.x, 1.0 - gl_PointCoord.y, 1 ) ).xy;
	#endif
#endif
#ifdef USE_MAP
	diffuseColor *= texture2D( map, uv );
#endif
#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, uv ).g;
#endif`,N0=`#if defined( USE_POINTS_UV )
	varying vec2 vUv;
#else
	#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
		uniform mat3 uvTransform;
	#endif
#endif
#ifdef USE_MAP
	uniform sampler2D map;
#endif
#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,B0=`float metalnessFactor = metalness;
#ifdef USE_METALNESSMAP
	vec4 texelMetalness = texture2D( metalnessMap, vMetalnessMapUv );
	metalnessFactor *= texelMetalness.b;
#endif`,O0=`#ifdef USE_METALNESSMAP
	uniform sampler2D metalnessMap;
#endif`,z0=`#ifdef USE_INSTANCING_MORPH
	float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	float morphTargetBaseInfluence = texelFetch( morphTexture, ivec2( 0, gl_InstanceID ), 0 ).r;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		morphTargetInfluences[i] =  texelFetch( morphTexture, ivec2( i + 1, gl_InstanceID ), 0 ).r;
	}
#endif`,H0=`#if defined( USE_MORPHCOLORS )
	vColor *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		#if defined( USE_COLOR_ALPHA )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ) * morphTargetInfluences[ i ];
		#elif defined( USE_COLOR )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ).rgb * morphTargetInfluences[ i ];
		#endif
	}
#endif`,V0=`#ifdef USE_MORPHNORMALS
	objectNormal *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) objectNormal += getMorph( gl_VertexID, i, 1 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,G0=`#ifdef USE_MORPHTARGETS
	#ifndef USE_INSTANCING_MORPH
		uniform float morphTargetBaseInfluence;
		uniform float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	#endif
	uniform sampler2DArray morphTargetsTexture;
	uniform ivec2 morphTargetsTextureSize;
	vec4 getMorph( const in int vertexIndex, const in int morphTargetIndex, const in int offset ) {
		int texelIndex = vertexIndex * MORPHTARGETS_TEXTURE_STRIDE + offset;
		int y = texelIndex / morphTargetsTextureSize.x;
		int x = texelIndex - y * morphTargetsTextureSize.x;
		ivec3 morphUV = ivec3( x, y, morphTargetIndex );
		return texelFetch( morphTargetsTexture, morphUV, 0 );
	}
#endif`,W0=`#ifdef USE_MORPHTARGETS
	transformed *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) transformed += getMorph( gl_VertexID, i, 0 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,q0=`float faceDirection = gl_FrontFacing ? 1.0 : - 1.0;
#ifdef FLAT_SHADED
	vec3 fdx = dFdx( vViewPosition );
	vec3 fdy = dFdy( vViewPosition );
	vec3 normal = normalize( cross( fdx, fdy ) );
#else
	vec3 normal = normalize( vNormal );
	#ifdef DOUBLE_SIDED
		normal *= faceDirection;
	#endif
#endif
#if defined( USE_NORMALMAP_TANGENTSPACE ) || defined( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY )
	#ifdef USE_TANGENT
		mat3 tbn = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn = getTangentFrame( - vViewPosition, normal,
		#if defined( USE_NORMALMAP )
			vNormalMapUv
		#elif defined( USE_CLEARCOAT_NORMALMAP )
			vClearcoatNormalMapUv
		#else
			vUv
		#endif
		);
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn[0] *= faceDirection;
		tbn[1] *= faceDirection;
	#endif
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	#ifdef USE_TANGENT
		mat3 tbn2 = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn2 = getTangentFrame( - vViewPosition, normal, vClearcoatNormalMapUv );
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn2[0] *= faceDirection;
		tbn2[1] *= faceDirection;
	#endif
#endif
vec3 nonPerturbedNormal = normal;`,X0=`#ifdef USE_NORMALMAP_OBJECTSPACE
	normal = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	#ifdef FLIP_SIDED
		normal = - normal;
	#endif
	#ifdef DOUBLE_SIDED
		normal = normal * faceDirection;
	#endif
	normal = normalize( normalMatrix * normal );
#elif defined( USE_NORMALMAP_TANGENTSPACE )
	vec3 mapN = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	mapN.xy *= normalScale;
	normal = normalize( tbn * mapN );
#elif defined( USE_BUMPMAP )
	normal = perturbNormalArb( - vViewPosition, normal, dHdxy_fwd(), faceDirection );
#endif`,$0=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,Y0=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,Z0=`#ifndef FLAT_SHADED
	vNormal = normalize( transformedNormal );
	#ifdef USE_TANGENT
		vTangent = normalize( transformedTangent );
		vBitangent = normalize( cross( vNormal, vTangent ) * tangent.w );
	#endif
#endif`,J0=`#ifdef USE_NORMALMAP
	uniform sampler2D normalMap;
	uniform vec2 normalScale;
#endif
#ifdef USE_NORMALMAP_OBJECTSPACE
	uniform mat3 normalMatrix;
#endif
#if ! defined ( USE_TANGENT ) && ( defined ( USE_NORMALMAP_TANGENTSPACE ) || defined ( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY ) )
	mat3 getTangentFrame( vec3 eye_pos, vec3 surf_norm, vec2 uv ) {
		vec3 q0 = dFdx( eye_pos.xyz );
		vec3 q1 = dFdy( eye_pos.xyz );
		vec2 st0 = dFdx( uv.st );
		vec2 st1 = dFdy( uv.st );
		vec3 N = surf_norm;
		vec3 q1perp = cross( q1, N );
		vec3 q0perp = cross( N, q0 );
		vec3 T = q1perp * st0.x + q0perp * st1.x;
		vec3 B = q1perp * st0.y + q0perp * st1.y;
		float det = max( dot( T, T ), dot( B, B ) );
		float scale = ( det == 0.0 ) ? 0.0 : inversesqrt( det );
		return mat3( T * scale, B * scale, N );
	}
#endif`,K0=`#ifdef USE_CLEARCOAT
	vec3 clearcoatNormal = nonPerturbedNormal;
#endif`,Q0=`#ifdef USE_CLEARCOAT_NORMALMAP
	vec3 clearcoatMapN = texture2D( clearcoatNormalMap, vClearcoatNormalMapUv ).xyz * 2.0 - 1.0;
	clearcoatMapN.xy *= clearcoatNormalScale;
	clearcoatNormal = normalize( tbn2 * clearcoatMapN );
#endif`,j0=`#ifdef USE_CLEARCOATMAP
	uniform sampler2D clearcoatMap;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform sampler2D clearcoatNormalMap;
	uniform vec2 clearcoatNormalScale;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform sampler2D clearcoatRoughnessMap;
#endif`,tg=`#ifdef USE_IRIDESCENCEMAP
	uniform sampler2D iridescenceMap;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform sampler2D iridescenceThicknessMap;
#endif`,eg=`#ifdef OPAQUE
diffuseColor.a = 1.0;
#endif
#ifdef USE_TRANSMISSION
diffuseColor.a *= material.transmissionAlpha;
#endif
gl_FragColor = vec4( outgoingLight, diffuseColor.a );`,ng=`vec3 packNormalToRGB( const in vec3 normal ) {
	return normalize( normal ) * 0.5 + 0.5;
}
vec3 unpackRGBToNormal( const in vec3 rgb ) {
	return 2.0 * rgb.xyz - 1.0;
}
const float PackUpscale = 256. / 255.;const float UnpackDownscale = 255. / 256.;const float ShiftRight8 = 1. / 256.;
const float Inv255 = 1. / 255.;
const vec4 PackFactors = vec4( 1.0, 256.0, 256.0 * 256.0, 256.0 * 256.0 * 256.0 );
const vec2 UnpackFactors2 = vec2( UnpackDownscale, 1.0 / PackFactors.g );
const vec3 UnpackFactors3 = vec3( UnpackDownscale / PackFactors.rg, 1.0 / PackFactors.b );
const vec4 UnpackFactors4 = vec4( UnpackDownscale / PackFactors.rgb, 1.0 / PackFactors.a );
vec4 packDepthToRGBA( const in float v ) {
	if( v <= 0.0 )
		return vec4( 0., 0., 0., 0. );
	if( v >= 1.0 )
		return vec4( 1., 1., 1., 1. );
	float vuf;
	float af = modf( v * PackFactors.a, vuf );
	float bf = modf( vuf * ShiftRight8, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec4( vuf * Inv255, gf * PackUpscale, bf * PackUpscale, af );
}
vec3 packDepthToRGB( const in float v ) {
	if( v <= 0.0 )
		return vec3( 0., 0., 0. );
	if( v >= 1.0 )
		return vec3( 1., 1., 1. );
	float vuf;
	float bf = modf( v * PackFactors.b, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec3( vuf * Inv255, gf * PackUpscale, bf );
}
vec2 packDepthToRG( const in float v ) {
	if( v <= 0.0 )
		return vec2( 0., 0. );
	if( v >= 1.0 )
		return vec2( 1., 1. );
	float vuf;
	float gf = modf( v * 256., vuf );
	return vec2( vuf * Inv255, gf );
}
float unpackRGBAToDepth( const in vec4 v ) {
	return dot( v, UnpackFactors4 );
}
float unpackRGBToDepth( const in vec3 v ) {
	return dot( v, UnpackFactors3 );
}
float unpackRGToDepth( const in vec2 v ) {
	return v.r * UnpackFactors2.r + v.g * UnpackFactors2.g;
}
vec4 pack2HalfToRGBA( const in vec2 v ) {
	vec4 r = vec4( v.x, fract( v.x * 255.0 ), v.y, fract( v.y * 255.0 ) );
	return vec4( r.x - r.y / 255.0, r.y, r.z - r.w / 255.0, r.w );
}
vec2 unpackRGBATo2Half( const in vec4 v ) {
	return vec2( v.x + ( v.y / 255.0 ), v.z + ( v.w / 255.0 ) );
}
float viewZToOrthographicDepth( const in float viewZ, const in float near, const in float far ) {
	return ( viewZ + near ) / ( near - far );
}
float orthographicDepthToViewZ( const in float depth, const in float near, const in float far ) {
	return depth * ( near - far ) - near;
}
float viewZToPerspectiveDepth( const in float viewZ, const in float near, const in float far ) {
	return ( ( near + viewZ ) * far ) / ( ( far - near ) * viewZ );
}
float perspectiveDepthToViewZ( const in float depth, const in float near, const in float far ) {
	return ( near * far ) / ( ( far - near ) * depth - far );
}`,ig=`#ifdef PREMULTIPLIED_ALPHA
	gl_FragColor.rgb *= gl_FragColor.a;
#endif`,sg=`vec4 mvPosition = vec4( transformed, 1.0 );
#ifdef USE_BATCHING
	mvPosition = batchingMatrix * mvPosition;
#endif
#ifdef USE_INSTANCING
	mvPosition = instanceMatrix * mvPosition;
#endif
mvPosition = modelViewMatrix * mvPosition;
gl_Position = projectionMatrix * mvPosition;`,rg=`#ifdef DITHERING
	gl_FragColor.rgb = dithering( gl_FragColor.rgb );
#endif`,ag=`#ifdef DITHERING
	vec3 dithering( vec3 color ) {
		float grid_position = rand( gl_FragCoord.xy );
		vec3 dither_shift_RGB = vec3( 0.25 / 255.0, -0.25 / 255.0, 0.25 / 255.0 );
		dither_shift_RGB = mix( 2.0 * dither_shift_RGB, -2.0 * dither_shift_RGB, grid_position );
		return color + dither_shift_RGB;
	}
#endif`,og=`float roughnessFactor = roughness;
#ifdef USE_ROUGHNESSMAP
	vec4 texelRoughness = texture2D( roughnessMap, vRoughnessMapUv );
	roughnessFactor *= texelRoughness.g;
#endif`,lg=`#ifdef USE_ROUGHNESSMAP
	uniform sampler2D roughnessMap;
#endif`,cg=`#if NUM_SPOT_LIGHT_COORDS > 0
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#if NUM_SPOT_LIGHT_MAPS > 0
	uniform sampler2D spotLightMap[ NUM_SPOT_LIGHT_MAPS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		uniform sampler2D directionalShadowMap[ NUM_DIR_LIGHT_SHADOWS ];
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		uniform sampler2D spotShadowMap[ NUM_SPOT_LIGHT_SHADOWS ];
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		uniform sampler2D pointShadowMap[ NUM_POINT_LIGHT_SHADOWS ];
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
	float texture2DCompare( sampler2D depths, vec2 uv, float compare ) {
		return step( compare, unpackRGBAToDepth( texture2D( depths, uv ) ) );
	}
	vec2 texture2DDistribution( sampler2D shadow, vec2 uv ) {
		return unpackRGBATo2Half( texture2D( shadow, uv ) );
	}
	float VSMShadow (sampler2D shadow, vec2 uv, float compare ){
		float occlusion = 1.0;
		vec2 distribution = texture2DDistribution( shadow, uv );
		float hard_shadow = step( compare , distribution.x );
		if (hard_shadow != 1.0 ) {
			float distance = compare - distribution.x ;
			float variance = max( 0.00000, distribution.y * distribution.y );
			float softness_probability = variance / (variance + distance * distance );			softness_probability = clamp( ( softness_probability - 0.3 ) / ( 0.95 - 0.3 ), 0.0, 1.0 );			occlusion = clamp( max( hard_shadow, softness_probability ), 0.0, 1.0 );
		}
		return occlusion;
	}
	float getShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord ) {
		float shadow = 1.0;
		shadowCoord.xyz /= shadowCoord.w;
		shadowCoord.z += shadowBias;
		bool inFrustum = shadowCoord.x >= 0.0 && shadowCoord.x <= 1.0 && shadowCoord.y >= 0.0 && shadowCoord.y <= 1.0;
		bool frustumTest = inFrustum && shadowCoord.z <= 1.0;
		if ( frustumTest ) {
		#if defined( SHADOWMAP_TYPE_PCF )
			vec2 texelSize = vec2( 1.0 ) / shadowMapSize;
			float dx0 = - texelSize.x * shadowRadius;
			float dy0 = - texelSize.y * shadowRadius;
			float dx1 = + texelSize.x * shadowRadius;
			float dy1 = + texelSize.y * shadowRadius;
			float dx2 = dx0 / 2.0;
			float dy2 = dy0 / 2.0;
			float dx3 = dx1 / 2.0;
			float dy3 = dy1 / 2.0;
			shadow = (
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy, shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, dy1 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy1 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, dy1 ), shadowCoord.z )
			) * ( 1.0 / 17.0 );
		#elif defined( SHADOWMAP_TYPE_PCF_SOFT )
			vec2 texelSize = vec2( 1.0 ) / shadowMapSize;
			float dx = texelSize.x;
			float dy = texelSize.y;
			vec2 uv = shadowCoord.xy;
			vec2 f = fract( uv * shadowMapSize + 0.5 );
			uv -= f * texelSize;
			shadow = (
				texture2DCompare( shadowMap, uv, shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + vec2( dx, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + vec2( 0.0, dy ), shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + texelSize, shadowCoord.z ) +
				mix( texture2DCompare( shadowMap, uv + vec2( -dx, 0.0 ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, 0.0 ), shadowCoord.z ),
					 f.x ) +
				mix( texture2DCompare( shadowMap, uv + vec2( -dx, dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, dy ), shadowCoord.z ),
					 f.x ) +
				mix( texture2DCompare( shadowMap, uv + vec2( 0.0, -dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 0.0, 2.0 * dy ), shadowCoord.z ),
					 f.y ) +
				mix( texture2DCompare( shadowMap, uv + vec2( dx, -dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( dx, 2.0 * dy ), shadowCoord.z ),
					 f.y ) +
				mix( mix( texture2DCompare( shadowMap, uv + vec2( -dx, -dy ), shadowCoord.z ),
						  texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, -dy ), shadowCoord.z ),
						  f.x ),
					 mix( texture2DCompare( shadowMap, uv + vec2( -dx, 2.0 * dy ), shadowCoord.z ),
						  texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, 2.0 * dy ), shadowCoord.z ),
						  f.x ),
					 f.y )
			) * ( 1.0 / 9.0 );
		#elif defined( SHADOWMAP_TYPE_VSM )
			shadow = VSMShadow( shadowMap, shadowCoord.xy, shadowCoord.z );
		#else
			shadow = texture2DCompare( shadowMap, shadowCoord.xy, shadowCoord.z );
		#endif
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
	vec2 cubeToUV( vec3 v, float texelSizeY ) {
		vec3 absV = abs( v );
		float scaleToCube = 1.0 / max( absV.x, max( absV.y, absV.z ) );
		absV *= scaleToCube;
		v *= scaleToCube * ( 1.0 - 2.0 * texelSizeY );
		vec2 planar = v.xy;
		float almostATexel = 1.5 * texelSizeY;
		float almostOne = 1.0 - almostATexel;
		if ( absV.z >= almostOne ) {
			if ( v.z > 0.0 )
				planar.x = 4.0 - v.x;
		} else if ( absV.x >= almostOne ) {
			float signX = sign( v.x );
			planar.x = v.z * signX + 2.0 * signX;
		} else if ( absV.y >= almostOne ) {
			float signY = sign( v.y );
			planar.x = v.x + 2.0 * signY + 2.0;
			planar.y = v.z * signY - 2.0;
		}
		return vec2( 0.125, 0.25 ) * planar + vec2( 0.375, 0.75 );
	}
	float getPointShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord, float shadowCameraNear, float shadowCameraFar ) {
		float shadow = 1.0;
		vec3 lightToPosition = shadowCoord.xyz;
		
		float lightToPositionLength = length( lightToPosition );
		if ( lightToPositionLength - shadowCameraFar <= 0.0 && lightToPositionLength - shadowCameraNear >= 0.0 ) {
			float dp = ( lightToPositionLength - shadowCameraNear ) / ( shadowCameraFar - shadowCameraNear );			dp += shadowBias;
			vec3 bd3D = normalize( lightToPosition );
			vec2 texelSize = vec2( 1.0 ) / ( shadowMapSize * vec2( 4.0, 2.0 ) );
			#if defined( SHADOWMAP_TYPE_PCF ) || defined( SHADOWMAP_TYPE_PCF_SOFT ) || defined( SHADOWMAP_TYPE_VSM )
				vec2 offset = vec2( - 1, 1 ) * shadowRadius * texelSize.y;
				shadow = (
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xyy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yyy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xyx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yyx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xxy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yxy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xxx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yxx, texelSize.y ), dp )
				) * ( 1.0 / 9.0 );
			#else
				shadow = texture2DCompare( shadowMap, cubeToUV( bd3D, texelSize.y ), dp );
			#endif
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
#endif`,hg=`#if NUM_SPOT_LIGHT_COORDS > 0
	uniform mat4 spotLightMatrix[ NUM_SPOT_LIGHT_COORDS ];
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		uniform mat4 directionalShadowMatrix[ NUM_DIR_LIGHT_SHADOWS ];
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		uniform mat4 pointShadowMatrix[ NUM_POINT_LIGHT_SHADOWS ];
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
#endif`,ug=`#if ( defined( USE_SHADOWMAP ) && ( NUM_DIR_LIGHT_SHADOWS > 0 || NUM_POINT_LIGHT_SHADOWS > 0 ) ) || ( NUM_SPOT_LIGHT_COORDS > 0 )
	vec3 shadowWorldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
	vec4 shadowWorldPosition;
#endif
#if defined( USE_SHADOWMAP )
	#if NUM_DIR_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * directionalLightShadows[ i ].shadowNormalBias, 0 );
			vDirectionalShadowCoord[ i ] = directionalShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * pointLightShadows[ i ].shadowNormalBias, 0 );
			vPointShadowCoord[ i ] = pointShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
#endif
#if NUM_SPOT_LIGHT_COORDS > 0
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_COORDS; i ++ ) {
		shadowWorldPosition = worldPosition;
		#if ( defined( USE_SHADOWMAP ) && UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
			shadowWorldPosition.xyz += shadowWorldNormal * spotLightShadows[ i ].shadowNormalBias;
		#endif
		vSpotLightCoord[ i ] = spotLightMatrix[ i ] * shadowWorldPosition;
	}
	#pragma unroll_loop_end
#endif`,dg=`float getShadowMask() {
	float shadow = 1.0;
	#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
		directionalLight = directionalLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( directionalShadowMap[ i ], directionalLight.shadowMapSize, directionalLight.shadowIntensity, directionalLight.shadowBias, directionalLight.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_SHADOWS; i ++ ) {
		spotLight = spotLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( spotShadowMap[ i ], spotLight.shadowMapSize, spotLight.shadowIntensity, spotLight.shadowBias, spotLight.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
	PointLightShadow pointLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
		pointLight = pointLightShadows[ i ];
		shadow *= receiveShadow ? getPointShadow( pointShadowMap[ i ], pointLight.shadowMapSize, pointLight.shadowIntensity, pointLight.shadowBias, pointLight.shadowRadius, vPointShadowCoord[ i ], pointLight.shadowCameraNear, pointLight.shadowCameraFar ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#endif
	return shadow;
}`,fg=`#ifdef USE_SKINNING
	mat4 boneMatX = getBoneMatrix( skinIndex.x );
	mat4 boneMatY = getBoneMatrix( skinIndex.y );
	mat4 boneMatZ = getBoneMatrix( skinIndex.z );
	mat4 boneMatW = getBoneMatrix( skinIndex.w );
#endif`,pg=`#ifdef USE_SKINNING
	uniform mat4 bindMatrix;
	uniform mat4 bindMatrixInverse;
	uniform highp sampler2D boneTexture;
	mat4 getBoneMatrix( const in float i ) {
		int size = textureSize( boneTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( boneTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( boneTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( boneTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( boneTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
#endif`,mg=`#ifdef USE_SKINNING
	vec4 skinVertex = bindMatrix * vec4( transformed, 1.0 );
	vec4 skinned = vec4( 0.0 );
	skinned += boneMatX * skinVertex * skinWeight.x;
	skinned += boneMatY * skinVertex * skinWeight.y;
	skinned += boneMatZ * skinVertex * skinWeight.z;
	skinned += boneMatW * skinVertex * skinWeight.w;
	transformed = ( bindMatrixInverse * skinned ).xyz;
#endif`,gg=`#ifdef USE_SKINNING
	mat4 skinMatrix = mat4( 0.0 );
	skinMatrix += skinWeight.x * boneMatX;
	skinMatrix += skinWeight.y * boneMatY;
	skinMatrix += skinWeight.z * boneMatZ;
	skinMatrix += skinWeight.w * boneMatW;
	skinMatrix = bindMatrixInverse * skinMatrix * bindMatrix;
	objectNormal = vec4( skinMatrix * vec4( objectNormal, 0.0 ) ).xyz;
	#ifdef USE_TANGENT
		objectTangent = vec4( skinMatrix * vec4( objectTangent, 0.0 ) ).xyz;
	#endif
#endif`,xg=`float specularStrength;
#ifdef USE_SPECULARMAP
	vec4 texelSpecular = texture2D( specularMap, vSpecularMapUv );
	specularStrength = texelSpecular.r;
#else
	specularStrength = 1.0;
#endif`,vg=`#ifdef USE_SPECULARMAP
	uniform sampler2D specularMap;
#endif`,yg=`#if defined( TONE_MAPPING )
	gl_FragColor.rgb = toneMapping( gl_FragColor.rgb );
#endif`,_g=`#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
uniform float toneMappingExposure;
vec3 LinearToneMapping( vec3 color ) {
	return saturate( toneMappingExposure * color );
}
vec3 ReinhardToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	return saturate( color / ( vec3( 1.0 ) + color ) );
}
vec3 CineonToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	color = max( vec3( 0.0 ), color - 0.004 );
	return pow( ( color * ( 6.2 * color + 0.5 ) ) / ( color * ( 6.2 * color + 1.7 ) + 0.06 ), vec3( 2.2 ) );
}
vec3 RRTAndODTFit( vec3 v ) {
	vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
	vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;
	return a / b;
}
vec3 ACESFilmicToneMapping( vec3 color ) {
	const mat3 ACESInputMat = mat3(
		vec3( 0.59719, 0.07600, 0.02840 ),		vec3( 0.35458, 0.90834, 0.13383 ),
		vec3( 0.04823, 0.01566, 0.83777 )
	);
	const mat3 ACESOutputMat = mat3(
		vec3(  1.60475, -0.10208, -0.00327 ),		vec3( -0.53108,  1.10813, -0.07276 ),
		vec3( -0.07367, -0.00605,  1.07602 )
	);
	color *= toneMappingExposure / 0.6;
	color = ACESInputMat * color;
	color = RRTAndODTFit( color );
	color = ACESOutputMat * color;
	return saturate( color );
}
const mat3 LINEAR_REC2020_TO_LINEAR_SRGB = mat3(
	vec3( 1.6605, - 0.1246, - 0.0182 ),
	vec3( - 0.5876, 1.1329, - 0.1006 ),
	vec3( - 0.0728, - 0.0083, 1.1187 )
);
const mat3 LINEAR_SRGB_TO_LINEAR_REC2020 = mat3(
	vec3( 0.6274, 0.0691, 0.0164 ),
	vec3( 0.3293, 0.9195, 0.0880 ),
	vec3( 0.0433, 0.0113, 0.8956 )
);
vec3 agxDefaultContrastApprox( vec3 x ) {
	vec3 x2 = x * x;
	vec3 x4 = x2 * x2;
	return + 15.5 * x4 * x2
		- 40.14 * x4 * x
		+ 31.96 * x4
		- 6.868 * x2 * x
		+ 0.4298 * x2
		+ 0.1191 * x
		- 0.00232;
}
vec3 AgXToneMapping( vec3 color ) {
	const mat3 AgXInsetMatrix = mat3(
		vec3( 0.856627153315983, 0.137318972929847, 0.11189821299995 ),
		vec3( 0.0951212405381588, 0.761241990602591, 0.0767994186031903 ),
		vec3( 0.0482516061458583, 0.101439036467562, 0.811302368396859 )
	);
	const mat3 AgXOutsetMatrix = mat3(
		vec3( 1.1271005818144368, - 0.1413297634984383, - 0.14132976349843826 ),
		vec3( - 0.11060664309660323, 1.157823702216272, - 0.11060664309660294 ),
		vec3( - 0.016493938717834573, - 0.016493938717834257, 1.2519364065950405 )
	);
	const float AgxMinEv = - 12.47393;	const float AgxMaxEv = 4.026069;
	color *= toneMappingExposure;
	color = LINEAR_SRGB_TO_LINEAR_REC2020 * color;
	color = AgXInsetMatrix * color;
	color = max( color, 1e-10 );	color = log2( color );
	color = ( color - AgxMinEv ) / ( AgxMaxEv - AgxMinEv );
	color = clamp( color, 0.0, 1.0 );
	color = agxDefaultContrastApprox( color );
	color = AgXOutsetMatrix * color;
	color = pow( max( vec3( 0.0 ), color ), vec3( 2.2 ) );
	color = LINEAR_REC2020_TO_LINEAR_SRGB * color;
	color = clamp( color, 0.0, 1.0 );
	return color;
}
vec3 NeutralToneMapping( vec3 color ) {
	const float StartCompression = 0.8 - 0.04;
	const float Desaturation = 0.15;
	color *= toneMappingExposure;
	float x = min( color.r, min( color.g, color.b ) );
	float offset = x < 0.08 ? x - 6.25 * x * x : 0.04;
	color -= offset;
	float peak = max( color.r, max( color.g, color.b ) );
	if ( peak < StartCompression ) return color;
	float d = 1. - StartCompression;
	float newPeak = 1. - d * d / ( peak + d - StartCompression );
	color *= newPeak / peak;
	float g = 1. - 1. / ( Desaturation * ( peak - newPeak ) + 1. );
	return mix( color, vec3( newPeak ), g );
}
vec3 CustomToneMapping( vec3 color ) { return color; }`,bg=`#ifdef USE_TRANSMISSION
	material.transmission = transmission;
	material.transmissionAlpha = 1.0;
	material.thickness = thickness;
	material.attenuationDistance = attenuationDistance;
	material.attenuationColor = attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		material.transmission *= texture2D( transmissionMap, vTransmissionMapUv ).r;
	#endif
	#ifdef USE_THICKNESSMAP
		material.thickness *= texture2D( thicknessMap, vThicknessMapUv ).g;
	#endif
	vec3 pos = vWorldPosition;
	vec3 v = normalize( cameraPosition - pos );
	vec3 n = inverseTransformDirection( normal, viewMatrix );
	vec4 transmitted = getIBLVolumeRefraction(
		n, v, material.roughness, material.diffuseColor, material.specularColor, material.specularF90,
		pos, modelMatrix, viewMatrix, projectionMatrix, material.dispersion, material.ior, material.thickness,
		material.attenuationColor, material.attenuationDistance );
	material.transmissionAlpha = mix( material.transmissionAlpha, transmitted.a, material.transmission );
	totalDiffuse = mix( totalDiffuse, transmitted.rgb, material.transmission );
#endif`,Mg=`#ifdef USE_TRANSMISSION
	uniform float transmission;
	uniform float thickness;
	uniform float attenuationDistance;
	uniform vec3 attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		uniform sampler2D transmissionMap;
	#endif
	#ifdef USE_THICKNESSMAP
		uniform sampler2D thicknessMap;
	#endif
	uniform vec2 transmissionSamplerSize;
	uniform sampler2D transmissionSamplerMap;
	uniform mat4 modelMatrix;
	uniform mat4 projectionMatrix;
	varying vec3 vWorldPosition;
	float w0( float a ) {
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - a + 3.0 ) - 3.0 ) + 1.0 );
	}
	float w1( float a ) {
		return ( 1.0 / 6.0 ) * ( a *  a * ( 3.0 * a - 6.0 ) + 4.0 );
	}
	float w2( float a ){
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - 3.0 * a + 3.0 ) + 3.0 ) + 1.0 );
	}
	float w3( float a ) {
		return ( 1.0 / 6.0 ) * ( a * a * a );
	}
	float g0( float a ) {
		return w0( a ) + w1( a );
	}
	float g1( float a ) {
		return w2( a ) + w3( a );
	}
	float h0( float a ) {
		return - 1.0 + w1( a ) / ( w0( a ) + w1( a ) );
	}
	float h1( float a ) {
		return 1.0 + w3( a ) / ( w2( a ) + w3( a ) );
	}
	vec4 bicubic( sampler2D tex, vec2 uv, vec4 texelSize, float lod ) {
		uv = uv * texelSize.zw + 0.5;
		vec2 iuv = floor( uv );
		vec2 fuv = fract( uv );
		float g0x = g0( fuv.x );
		float g1x = g1( fuv.x );
		float h0x = h0( fuv.x );
		float h1x = h1( fuv.x );
		float h0y = h0( fuv.y );
		float h1y = h1( fuv.y );
		vec2 p0 = ( vec2( iuv.x + h0x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p1 = ( vec2( iuv.x + h1x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p2 = ( vec2( iuv.x + h0x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		vec2 p3 = ( vec2( iuv.x + h1x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		return g0( fuv.y ) * ( g0x * textureLod( tex, p0, lod ) + g1x * textureLod( tex, p1, lod ) ) +
			g1( fuv.y ) * ( g0x * textureLod( tex, p2, lod ) + g1x * textureLod( tex, p3, lod ) );
	}
	vec4 textureBicubic( sampler2D sampler, vec2 uv, float lod ) {
		vec2 fLodSize = vec2( textureSize( sampler, int( lod ) ) );
		vec2 cLodSize = vec2( textureSize( sampler, int( lod + 1.0 ) ) );
		vec2 fLodSizeInv = 1.0 / fLodSize;
		vec2 cLodSizeInv = 1.0 / cLodSize;
		vec4 fSample = bicubic( sampler, uv, vec4( fLodSizeInv, fLodSize ), floor( lod ) );
		vec4 cSample = bicubic( sampler, uv, vec4( cLodSizeInv, cLodSize ), ceil( lod ) );
		return mix( fSample, cSample, fract( lod ) );
	}
	vec3 getVolumeTransmissionRay( const in vec3 n, const in vec3 v, const in float thickness, const in float ior, const in mat4 modelMatrix ) {
		vec3 refractionVector = refract( - v, normalize( n ), 1.0 / ior );
		vec3 modelScale;
		modelScale.x = length( vec3( modelMatrix[ 0 ].xyz ) );
		modelScale.y = length( vec3( modelMatrix[ 1 ].xyz ) );
		modelScale.z = length( vec3( modelMatrix[ 2 ].xyz ) );
		return normalize( refractionVector ) * thickness * modelScale;
	}
	float applyIorToRoughness( const in float roughness, const in float ior ) {
		return roughness * clamp( ior * 2.0 - 2.0, 0.0, 1.0 );
	}
	vec4 getTransmissionSample( const in vec2 fragCoord, const in float roughness, const in float ior ) {
		float lod = log2( transmissionSamplerSize.x ) * applyIorToRoughness( roughness, ior );
		return textureBicubic( transmissionSamplerMap, fragCoord.xy, lod );
	}
	vec3 volumeAttenuation( const in float transmissionDistance, const in vec3 attenuationColor, const in float attenuationDistance ) {
		if ( isinf( attenuationDistance ) ) {
			return vec3( 1.0 );
		} else {
			vec3 attenuationCoefficient = -log( attenuationColor ) / attenuationDistance;
			vec3 transmittance = exp( - attenuationCoefficient * transmissionDistance );			return transmittance;
		}
	}
	vec4 getIBLVolumeRefraction( const in vec3 n, const in vec3 v, const in float roughness, const in vec3 diffuseColor,
		const in vec3 specularColor, const in float specularF90, const in vec3 position, const in mat4 modelMatrix,
		const in mat4 viewMatrix, const in mat4 projMatrix, const in float dispersion, const in float ior, const in float thickness,
		const in vec3 attenuationColor, const in float attenuationDistance ) {
		vec4 transmittedLight;
		vec3 transmittance;
		#ifdef USE_DISPERSION
			float halfSpread = ( ior - 1.0 ) * 0.025 * dispersion;
			vec3 iors = vec3( ior - halfSpread, ior, ior + halfSpread );
			for ( int i = 0; i < 3; i ++ ) {
				vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, iors[ i ], modelMatrix );
				vec3 refractedRayExit = position + transmissionRay;
		
				vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
				vec2 refractionCoords = ndcPos.xy / ndcPos.w;
				refractionCoords += 1.0;
				refractionCoords /= 2.0;
		
				vec4 transmissionSample = getTransmissionSample( refractionCoords, roughness, iors[ i ] );
				transmittedLight[ i ] = transmissionSample[ i ];
				transmittedLight.a += transmissionSample.a;
				transmittance[ i ] = diffuseColor[ i ] * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance )[ i ];
			}
			transmittedLight.a /= 3.0;
		
		#else
		
			vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, ior, modelMatrix );
			vec3 refractedRayExit = position + transmissionRay;
			vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
			vec2 refractionCoords = ndcPos.xy / ndcPos.w;
			refractionCoords += 1.0;
			refractionCoords /= 2.0;
			transmittedLight = getTransmissionSample( refractionCoords, roughness, ior );
			transmittance = diffuseColor * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance );
		
		#endif
		vec3 attenuatedColor = transmittance * transmittedLight.rgb;
		vec3 F = EnvironmentBRDF( n, v, specularColor, specularF90, roughness );
		float transmittanceFactor = ( transmittance.r + transmittance.g + transmittance.b ) / 3.0;
		return vec4( ( 1.0 - F ) * attenuatedColor, 1.0 - ( 1.0 - transmittedLight.a ) * transmittanceFactor );
	}
#endif`,Sg=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_SPECULARMAP
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,wg=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	uniform mat3 mapTransform;
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	uniform mat3 alphaMapTransform;
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	uniform mat3 lightMapTransform;
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	uniform mat3 aoMapTransform;
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	uniform mat3 bumpMapTransform;
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	uniform mat3 normalMapTransform;
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_DISPLACEMENTMAP
	uniform mat3 displacementMapTransform;
	varying vec2 vDisplacementMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	uniform mat3 emissiveMapTransform;
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	uniform mat3 metalnessMapTransform;
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	uniform mat3 roughnessMapTransform;
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	uniform mat3 anisotropyMapTransform;
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	uniform mat3 clearcoatMapTransform;
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform mat3 clearcoatNormalMapTransform;
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform mat3 clearcoatRoughnessMapTransform;
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	uniform mat3 sheenColorMapTransform;
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	uniform mat3 sheenRoughnessMapTransform;
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	uniform mat3 iridescenceMapTransform;
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform mat3 iridescenceThicknessMapTransform;
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SPECULARMAP
	uniform mat3 specularMapTransform;
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	uniform mat3 specularColorMapTransform;
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	uniform mat3 specularIntensityMapTransform;
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,Eg=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	vUv = vec3( uv, 1 ).xy;
#endif
#ifdef USE_MAP
	vMapUv = ( mapTransform * vec3( MAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ALPHAMAP
	vAlphaMapUv = ( alphaMapTransform * vec3( ALPHAMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_LIGHTMAP
	vLightMapUv = ( lightMapTransform * vec3( LIGHTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_AOMAP
	vAoMapUv = ( aoMapTransform * vec3( AOMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_BUMPMAP
	vBumpMapUv = ( bumpMapTransform * vec3( BUMPMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_NORMALMAP
	vNormalMapUv = ( normalMapTransform * vec3( NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_DISPLACEMENTMAP
	vDisplacementMapUv = ( displacementMapTransform * vec3( DISPLACEMENTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_EMISSIVEMAP
	vEmissiveMapUv = ( emissiveMapTransform * vec3( EMISSIVEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_METALNESSMAP
	vMetalnessMapUv = ( metalnessMapTransform * vec3( METALNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ROUGHNESSMAP
	vRoughnessMapUv = ( roughnessMapTransform * vec3( ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ANISOTROPYMAP
	vAnisotropyMapUv = ( anisotropyMapTransform * vec3( ANISOTROPYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOATMAP
	vClearcoatMapUv = ( clearcoatMapTransform * vec3( CLEARCOATMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	vClearcoatNormalMapUv = ( clearcoatNormalMapTransform * vec3( CLEARCOAT_NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	vClearcoatRoughnessMapUv = ( clearcoatRoughnessMapTransform * vec3( CLEARCOAT_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCEMAP
	vIridescenceMapUv = ( iridescenceMapTransform * vec3( IRIDESCENCEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	vIridescenceThicknessMapUv = ( iridescenceThicknessMapTransform * vec3( IRIDESCENCE_THICKNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_COLORMAP
	vSheenColorMapUv = ( sheenColorMapTransform * vec3( SHEEN_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	vSheenRoughnessMapUv = ( sheenRoughnessMapTransform * vec3( SHEEN_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULARMAP
	vSpecularMapUv = ( specularMapTransform * vec3( SPECULARMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_COLORMAP
	vSpecularColorMapUv = ( specularColorMapTransform * vec3( SPECULAR_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	vSpecularIntensityMapUv = ( specularIntensityMapTransform * vec3( SPECULAR_INTENSITYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_TRANSMISSIONMAP
	vTransmissionMapUv = ( transmissionMapTransform * vec3( TRANSMISSIONMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_THICKNESSMAP
	vThicknessMapUv = ( thicknessMapTransform * vec3( THICKNESSMAP_UV, 1 ) ).xy;
#endif`,Tg=`#if defined( USE_ENVMAP ) || defined( DISTANCE ) || defined ( USE_SHADOWMAP ) || defined ( USE_TRANSMISSION ) || NUM_SPOT_LIGHT_COORDS > 0
	vec4 worldPosition = vec4( transformed, 1.0 );
	#ifdef USE_BATCHING
		worldPosition = batchingMatrix * worldPosition;
	#endif
	#ifdef USE_INSTANCING
		worldPosition = instanceMatrix * worldPosition;
	#endif
	worldPosition = modelMatrix * worldPosition;
#endif`,Ag=`varying vec2 vUv;
uniform mat3 uvTransform;
void main() {
	vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	gl_Position = vec4( position.xy, 1.0, 1.0 );
}`,Cg=`uniform sampler2D t2D;
uniform float backgroundIntensity;
varying vec2 vUv;
void main() {
	vec4 texColor = texture2D( t2D, vUv );
	#ifdef DECODE_VIDEO_TEXTURE
		texColor = vec4( mix( pow( texColor.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), texColor.rgb * 0.0773993808, vec3( lessThanEqual( texColor.rgb, vec3( 0.04045 ) ) ) ), texColor.w );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Rg=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,Pg=`#ifdef ENVMAP_TYPE_CUBE
	uniform samplerCube envMap;
#elif defined( ENVMAP_TYPE_CUBE_UV )
	uniform sampler2D envMap;
#endif
uniform float flipEnvMap;
uniform float backgroundBlurriness;
uniform float backgroundIntensity;
uniform mat3 backgroundRotation;
varying vec3 vWorldDirection;
#include <cube_uv_reflection_fragment>
void main() {
	#ifdef ENVMAP_TYPE_CUBE
		vec4 texColor = textureCube( envMap, backgroundRotation * vec3( flipEnvMap * vWorldDirection.x, vWorldDirection.yz ) );
	#elif defined( ENVMAP_TYPE_CUBE_UV )
		vec4 texColor = textureCubeUV( envMap, backgroundRotation * vWorldDirection, backgroundBlurriness );
	#else
		vec4 texColor = vec4( 0.0, 0.0, 0.0, 1.0 );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Ig=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,Fg=`uniform samplerCube tCube;
uniform float tFlip;
uniform float opacity;
varying vec3 vWorldDirection;
void main() {
	vec4 texColor = textureCube( tCube, vec3( tFlip * vWorldDirection.x, vWorldDirection.yz ) );
	gl_FragColor = texColor;
	gl_FragColor.a *= opacity;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Dg=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
varying vec2 vHighPrecisionZW;
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vHighPrecisionZW = gl_Position.zw;
}`,kg=`#if DEPTH_PACKING == 3200
	uniform float opacity;
#endif
#include <common>
#include <packing>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
varying vec2 vHighPrecisionZW;
void main() {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#if DEPTH_PACKING == 3200
		diffuseColor.a = opacity;
	#endif
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <logdepthbuf_fragment>
	float fragCoordZ = 0.5 * vHighPrecisionZW[0] / vHighPrecisionZW[1] + 0.5;
	#if DEPTH_PACKING == 3200
		gl_FragColor = vec4( vec3( 1.0 - fragCoordZ ), opacity );
	#elif DEPTH_PACKING == 3201
		gl_FragColor = packDepthToRGBA( fragCoordZ );
	#elif DEPTH_PACKING == 3202
		gl_FragColor = vec4( packDepthToRGB( fragCoordZ ), 1.0 );
	#elif DEPTH_PACKING == 3203
		gl_FragColor = vec4( packDepthToRG( fragCoordZ ), 0.0, 1.0 );
	#endif
}`,Lg=`#define DISTANCE
varying vec3 vWorldPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <worldpos_vertex>
	#include <clipping_planes_vertex>
	vWorldPosition = worldPosition.xyz;
}`,Ug=`#define DISTANCE
uniform vec3 referencePosition;
uniform float nearDistance;
uniform float farDistance;
varying vec3 vWorldPosition;
#include <common>
#include <packing>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <clipping_planes_pars_fragment>
void main () {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	float dist = length( vWorldPosition - referencePosition );
	dist = ( dist - nearDistance ) / ( farDistance - nearDistance );
	dist = saturate( dist );
	gl_FragColor = packDepthToRGBA( dist );
}`,Ng=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
}`,Bg=`uniform sampler2D tEquirect;
varying vec3 vWorldDirection;
#include <common>
void main() {
	vec3 direction = normalize( vWorldDirection );
	vec2 sampleUV = equirectUv( direction );
	gl_FragColor = texture2D( tEquirect, sampleUV );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Og=`uniform float scale;
attribute float lineDistance;
varying float vLineDistance;
#include <common>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	vLineDistance = scale * lineDistance;
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,zg=`uniform vec3 diffuse;
uniform float opacity;
uniform float dashSize;
uniform float totalSize;
varying float vLineDistance;
#include <common>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	if ( mod( vLineDistance, totalSize ) > dashSize ) {
		discard;
	}
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,Hg=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#if defined ( USE_ENVMAP ) || defined ( USE_SKINNING )
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinbase_vertex>
		#include <skinnormal_vertex>
		#include <defaultnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <fog_vertex>
}`,Vg=`uniform vec3 diffuse;
uniform float opacity;
#ifndef FLAT_SHADED
	varying vec3 vNormal;
#endif
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		reflectedLight.indirectDiffuse += lightMapTexel.rgb * lightMapIntensity * RECIPROCAL_PI;
	#else
		reflectedLight.indirectDiffuse += vec3( 1.0 );
	#endif
	#include <aomap_fragment>
	reflectedLight.indirectDiffuse *= diffuseColor.rgb;
	vec3 outgoingLight = reflectedLight.indirectDiffuse;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,Gg=`#define LAMBERT
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,Wg=`#define LAMBERT
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_lambert_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_lambert_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,qg=`#define MATCAP
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <displacementmap_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
	vViewPosition = - mvPosition.xyz;
}`,Xg=`#define MATCAP
uniform vec3 diffuse;
uniform float opacity;
uniform sampler2D matcap;
varying vec3 vViewPosition;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	vec3 viewDir = normalize( vViewPosition );
	vec3 x = normalize( vec3( viewDir.z, 0.0, - viewDir.x ) );
	vec3 y = cross( viewDir, x );
	vec2 uv = vec2( dot( x, normal ), dot( y, normal ) ) * 0.495 + 0.5;
	#ifdef USE_MATCAP
		vec4 matcapColor = texture2D( matcap, uv );
	#else
		vec4 matcapColor = vec4( vec3( mix( 0.2, 0.8, uv.y ) ), 1.0 );
	#endif
	vec3 outgoingLight = diffuseColor.rgb * matcapColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,$g=`#define NORMAL
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	vViewPosition = - mvPosition.xyz;
#endif
}`,Yg=`#define NORMAL
uniform float opacity;
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <packing>
#include <uv_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( 0.0, 0.0, 0.0, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	gl_FragColor = vec4( packNormalToRGB( normal ), diffuseColor.a );
	#ifdef OPAQUE
		gl_FragColor.a = 1.0;
	#endif
}`,Zg=`#define PHONG
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,Jg=`#define PHONG
uniform vec3 diffuse;
uniform vec3 emissive;
uniform vec3 specular;
uniform float shininess;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_phong_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_phong_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + reflectedLight.directSpecular + reflectedLight.indirectSpecular + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,Kg=`#define STANDARD
varying vec3 vViewPosition;
#ifdef USE_TRANSMISSION
	varying vec3 vWorldPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
#ifdef USE_TRANSMISSION
	vWorldPosition = worldPosition.xyz;
#endif
}`,Qg=`#define STANDARD
#ifdef PHYSICAL
	#define IOR
	#define USE_SPECULAR
#endif
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float roughness;
uniform float metalness;
uniform float opacity;
#ifdef IOR
	uniform float ior;
#endif
#ifdef USE_SPECULAR
	uniform float specularIntensity;
	uniform vec3 specularColor;
	#ifdef USE_SPECULAR_COLORMAP
		uniform sampler2D specularColorMap;
	#endif
	#ifdef USE_SPECULAR_INTENSITYMAP
		uniform sampler2D specularIntensityMap;
	#endif
#endif
#ifdef USE_CLEARCOAT
	uniform float clearcoat;
	uniform float clearcoatRoughness;
#endif
#ifdef USE_DISPERSION
	uniform float dispersion;
#endif
#ifdef USE_IRIDESCENCE
	uniform float iridescence;
	uniform float iridescenceIOR;
	uniform float iridescenceThicknessMinimum;
	uniform float iridescenceThicknessMaximum;
#endif
#ifdef USE_SHEEN
	uniform vec3 sheenColor;
	uniform float sheenRoughness;
	#ifdef USE_SHEEN_COLORMAP
		uniform sampler2D sheenColorMap;
	#endif
	#ifdef USE_SHEEN_ROUGHNESSMAP
		uniform sampler2D sheenRoughnessMap;
	#endif
#endif
#ifdef USE_ANISOTROPY
	uniform vec2 anisotropyVector;
	#ifdef USE_ANISOTROPYMAP
		uniform sampler2D anisotropyMap;
	#endif
#endif
varying vec3 vViewPosition;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <iridescence_fragment>
#include <cube_uv_reflection_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_physical_pars_fragment>
#include <fog_pars_fragment>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_physical_pars_fragment>
#include <transmission_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <clearcoat_pars_fragment>
#include <iridescence_pars_fragment>
#include <roughnessmap_pars_fragment>
#include <metalnessmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <roughnessmap_fragment>
	#include <metalnessmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <clearcoat_normal_fragment_begin>
	#include <clearcoat_normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_physical_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 totalDiffuse = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse;
	vec3 totalSpecular = reflectedLight.directSpecular + reflectedLight.indirectSpecular;
	#include <transmission_fragment>
	vec3 outgoingLight = totalDiffuse + totalSpecular + totalEmissiveRadiance;
	#ifdef USE_SHEEN
		float sheenEnergyComp = 1.0 - 0.157 * max3( material.sheenColor );
		outgoingLight = outgoingLight * sheenEnergyComp + sheenSpecularDirect + sheenSpecularIndirect;
	#endif
	#ifdef USE_CLEARCOAT
		float dotNVcc = saturate( dot( geometryClearcoatNormal, geometryViewDir ) );
		vec3 Fcc = F_Schlick( material.clearcoatF0, material.clearcoatF90, dotNVcc );
		outgoingLight = outgoingLight * ( 1.0 - material.clearcoat * Fcc ) + ( clearcoatSpecularDirect + clearcoatSpecularIndirect ) * material.clearcoat;
	#endif
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,jg=`#define TOON
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,tx=`#define TOON
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <gradientmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_toon_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_toon_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,ex=`uniform float size;
uniform float scale;
#include <common>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
#ifdef USE_POINTS_UV
	varying vec2 vUv;
	uniform mat3 uvTransform;
#endif
void main() {
	#ifdef USE_POINTS_UV
		vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	#endif
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	gl_PointSize = size;
	#ifdef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) gl_PointSize *= ( scale / - mvPosition.z );
	#endif
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <fog_vertex>
}`,nx=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <color_pars_fragment>
#include <map_particle_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_particle_fragment>
	#include <color_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,ix=`#include <common>
#include <batching_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <shadowmap_pars_vertex>
void main() {
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,sx=`uniform vec3 color;
uniform float opacity;
#include <common>
#include <packing>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <logdepthbuf_pars_fragment>
#include <shadowmap_pars_fragment>
#include <shadowmask_pars_fragment>
void main() {
	#include <logdepthbuf_fragment>
	gl_FragColor = vec4( color, opacity * ( 1.0 - getShadowMask() ) );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
}`,rx=`uniform float rotation;
uniform vec2 center;
#include <common>
#include <uv_pars_vertex>
#include <fog_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	vec4 mvPosition = modelViewMatrix[ 3 ];
	vec2 scale = vec2( length( modelMatrix[ 0 ].xyz ), length( modelMatrix[ 1 ].xyz ) );
	#ifndef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) scale *= - mvPosition.z;
	#endif
	vec2 alignedPosition = ( position.xy - ( center - vec2( 0.5 ) ) ) * scale;
	vec2 rotatedPosition;
	rotatedPosition.x = cos( rotation ) * alignedPosition.x - sin( rotation ) * alignedPosition.y;
	rotatedPosition.y = sin( rotation ) * alignedPosition.x + cos( rotation ) * alignedPosition.y;
	mvPosition.xy += rotatedPosition;
	gl_Position = projectionMatrix * mvPosition;
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,ax=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
}`,te={alphahash_fragment:Cm,alphahash_pars_fragment:Rm,alphamap_fragment:Pm,alphamap_pars_fragment:Im,alphatest_fragment:Fm,alphatest_pars_fragment:Dm,aomap_fragment:km,aomap_pars_fragment:Lm,batching_pars_vertex:Um,batching_vertex:Nm,begin_vertex:Bm,beginnormal_vertex:Om,bsdfs:zm,iridescence_fragment:Hm,bumpmap_pars_fragment:Vm,clipping_planes_fragment:Gm,clipping_planes_pars_fragment:Wm,clipping_planes_pars_vertex:qm,clipping_planes_vertex:Xm,color_fragment:$m,color_pars_fragment:Ym,color_pars_vertex:Zm,color_vertex:Jm,common:Km,cube_uv_reflection_fragment:Qm,defaultnormal_vertex:jm,displacementmap_pars_vertex:t0,displacementmap_vertex:e0,emissivemap_fragment:n0,emissivemap_pars_fragment:i0,colorspace_fragment:s0,colorspace_pars_fragment:r0,envmap_fragment:a0,envmap_common_pars_fragment:o0,envmap_pars_fragment:l0,envmap_pars_vertex:c0,envmap_physical_pars_fragment:_0,envmap_vertex:h0,fog_vertex:u0,fog_pars_vertex:d0,fog_fragment:f0,fog_pars_fragment:p0,gradientmap_pars_fragment:m0,lightmap_pars_fragment:g0,lights_lambert_fragment:x0,lights_lambert_pars_fragment:v0,lights_pars_begin:y0,lights_toon_fragment:b0,lights_toon_pars_fragment:M0,lights_phong_fragment:S0,lights_phong_pars_fragment:w0,lights_physical_fragment:E0,lights_physical_pars_fragment:T0,lights_fragment_begin:A0,lights_fragment_maps:C0,lights_fragment_end:R0,logdepthbuf_fragment:P0,logdepthbuf_pars_fragment:I0,logdepthbuf_pars_vertex:F0,logdepthbuf_vertex:D0,map_fragment:k0,map_pars_fragment:L0,map_particle_fragment:U0,map_particle_pars_fragment:N0,metalnessmap_fragment:B0,metalnessmap_pars_fragment:O0,morphinstance_vertex:z0,morphcolor_vertex:H0,morphnormal_vertex:V0,morphtarget_pars_vertex:G0,morphtarget_vertex:W0,normal_fragment_begin:q0,normal_fragment_maps:X0,normal_pars_fragment:$0,normal_pars_vertex:Y0,normal_vertex:Z0,normalmap_pars_fragment:J0,clearcoat_normal_fragment_begin:K0,clearcoat_normal_fragment_maps:Q0,clearcoat_pars_fragment:j0,iridescence_pars_fragment:tg,opaque_fragment:eg,packing:ng,premultiplied_alpha_fragment:ig,project_vertex:sg,dithering_fragment:rg,dithering_pars_fragment:ag,roughnessmap_fragment:og,roughnessmap_pars_fragment:lg,shadowmap_pars_fragment:cg,shadowmap_pars_vertex:hg,shadowmap_vertex:ug,shadowmask_pars_fragment:dg,skinbase_vertex:fg,skinning_pars_vertex:pg,skinning_vertex:mg,skinnormal_vertex:gg,specularmap_fragment:xg,specularmap_pars_fragment:vg,tonemapping_fragment:yg,tonemapping_pars_fragment:_g,transmission_fragment:bg,transmission_pars_fragment:Mg,uv_pars_fragment:Sg,uv_pars_vertex:wg,uv_vertex:Eg,worldpos_vertex:Tg,background_vert:Ag,background_frag:Cg,backgroundCube_vert:Rg,backgroundCube_frag:Pg,cube_vert:Ig,cube_frag:Fg,depth_vert:Dg,depth_frag:kg,distanceRGBA_vert:Lg,distanceRGBA_frag:Ug,equirect_vert:Ng,equirect_frag:Bg,linedashed_vert:Og,linedashed_frag:zg,meshbasic_vert:Hg,meshbasic_frag:Vg,meshlambert_vert:Gg,meshlambert_frag:Wg,meshmatcap_vert:qg,meshmatcap_frag:Xg,meshnormal_vert:$g,meshnormal_frag:Yg,meshphong_vert:Zg,meshphong_frag:Jg,meshphysical_vert:Kg,meshphysical_frag:Qg,meshtoon_vert:jg,meshtoon_frag:tx,points_vert:ex,points_frag:nx,shadow_vert:ix,shadow_frag:sx,sprite_vert:rx,sprite_frag:ax},ht={common:{diffuse:{value:new Pt(16777215)},opacity:{value:1},map:{value:null},mapTransform:{value:new Kt},alphaMap:{value:null},alphaMapTransform:{value:new Kt},alphaTest:{value:0}},specularmap:{specularMap:{value:null},specularMapTransform:{value:new Kt}},envmap:{envMap:{value:null},envMapRotation:{value:new Kt},flipEnvMap:{value:-1},reflectivity:{value:1},ior:{value:1.5},refractionRatio:{value:.98}},aomap:{aoMap:{value:null},aoMapIntensity:{value:1},aoMapTransform:{value:new Kt}},lightmap:{lightMap:{value:null},lightMapIntensity:{value:1},lightMapTransform:{value:new Kt}},bumpmap:{bumpMap:{value:null},bumpMapTransform:{value:new Kt},bumpScale:{value:1}},normalmap:{normalMap:{value:null},normalMapTransform:{value:new Kt},normalScale:{value:new j(1,1)}},displacementmap:{displacementMap:{value:null},displacementMapTransform:{value:new Kt},displacementScale:{value:1},displacementBias:{value:0}},emissivemap:{emissiveMap:{value:null},emissiveMapTransform:{value:new Kt}},metalnessmap:{metalnessMap:{value:null},metalnessMapTransform:{value:new Kt}},roughnessmap:{roughnessMap:{value:null},roughnessMapTransform:{value:new Kt}},gradientmap:{gradientMap:{value:null}},fog:{fogDensity:{value:25e-5},fogNear:{value:1},fogFar:{value:2e3},fogColor:{value:new Pt(16777215)}},lights:{ambientLightColor:{value:[]},lightProbe:{value:[]},directionalLights:{value:[],properties:{direction:{},color:{}}},directionalLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},directionalShadowMap:{value:[]},directionalShadowMatrix:{value:[]},spotLights:{value:[],properties:{color:{},position:{},direction:{},distance:{},coneCos:{},penumbraCos:{},decay:{}}},spotLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},spotLightMap:{value:[]},spotShadowMap:{value:[]},spotLightMatrix:{value:[]},pointLights:{value:[],properties:{color:{},position:{},decay:{},distance:{}}},pointLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{},shadowCameraNear:{},shadowCameraFar:{}}},pointShadowMap:{value:[]},pointShadowMatrix:{value:[]},hemisphereLights:{value:[],properties:{direction:{},skyColor:{},groundColor:{}}},rectAreaLights:{value:[],properties:{color:{},position:{},width:{},height:{}}},ltc_1:{value:null},ltc_2:{value:null}},points:{diffuse:{value:new Pt(16777215)},opacity:{value:1},size:{value:1},scale:{value:1},map:{value:null},alphaMap:{value:null},alphaMapTransform:{value:new Kt},alphaTest:{value:0},uvTransform:{value:new Kt}},sprite:{diffuse:{value:new Pt(16777215)},opacity:{value:1},center:{value:new j(.5,.5)},rotation:{value:0},map:{value:null},mapTransform:{value:new Kt},alphaMap:{value:null},alphaMapTransform:{value:new Kt},alphaTest:{value:0}}},Yn={basic:{uniforms:rn([ht.common,ht.specularmap,ht.envmap,ht.aomap,ht.lightmap,ht.fog]),vertexShader:te.meshbasic_vert,fragmentShader:te.meshbasic_frag},lambert:{uniforms:rn([ht.common,ht.specularmap,ht.envmap,ht.aomap,ht.lightmap,ht.emissivemap,ht.bumpmap,ht.normalmap,ht.displacementmap,ht.fog,ht.lights,{emissive:{value:new Pt(0)}}]),vertexShader:te.meshlambert_vert,fragmentShader:te.meshlambert_frag},phong:{uniforms:rn([ht.common,ht.specularmap,ht.envmap,ht.aomap,ht.lightmap,ht.emissivemap,ht.bumpmap,ht.normalmap,ht.displacementmap,ht.fog,ht.lights,{emissive:{value:new Pt(0)},specular:{value:new Pt(1118481)},shininess:{value:30}}]),vertexShader:te.meshphong_vert,fragmentShader:te.meshphong_frag},standard:{uniforms:rn([ht.common,ht.envmap,ht.aomap,ht.lightmap,ht.emissivemap,ht.bumpmap,ht.normalmap,ht.displacementmap,ht.roughnessmap,ht.metalnessmap,ht.fog,ht.lights,{emissive:{value:new Pt(0)},roughness:{value:1},metalness:{value:0},envMapIntensity:{value:1}}]),vertexShader:te.meshphysical_vert,fragmentShader:te.meshphysical_frag},toon:{uniforms:rn([ht.common,ht.aomap,ht.lightmap,ht.emissivemap,ht.bumpmap,ht.normalmap,ht.displacementmap,ht.gradientmap,ht.fog,ht.lights,{emissive:{value:new Pt(0)}}]),vertexShader:te.meshtoon_vert,fragmentShader:te.meshtoon_frag},matcap:{uniforms:rn([ht.common,ht.bumpmap,ht.normalmap,ht.displacementmap,ht.fog,{matcap:{value:null}}]),vertexShader:te.meshmatcap_vert,fragmentShader:te.meshmatcap_frag},points:{uniforms:rn([ht.points,ht.fog]),vertexShader:te.points_vert,fragmentShader:te.points_frag},dashed:{uniforms:rn([ht.common,ht.fog,{scale:{value:1},dashSize:{value:1},totalSize:{value:2}}]),vertexShader:te.linedashed_vert,fragmentShader:te.linedashed_frag},depth:{uniforms:rn([ht.common,ht.displacementmap]),vertexShader:te.depth_vert,fragmentShader:te.depth_frag},normal:{uniforms:rn([ht.common,ht.bumpmap,ht.normalmap,ht.displacementmap,{opacity:{value:1}}]),vertexShader:te.meshnormal_vert,fragmentShader:te.meshnormal_frag},sprite:{uniforms:rn([ht.sprite,ht.fog]),vertexShader:te.sprite_vert,fragmentShader:te.sprite_frag},background:{uniforms:{uvTransform:{value:new Kt},t2D:{value:null},backgroundIntensity:{value:1}},vertexShader:te.background_vert,fragmentShader:te.background_frag},backgroundCube:{uniforms:{envMap:{value:null},flipEnvMap:{value:-1},backgroundBlurriness:{value:0},backgroundIntensity:{value:1},backgroundRotation:{value:new Kt}},vertexShader:te.backgroundCube_vert,fragmentShader:te.backgroundCube_frag},cube:{uniforms:{tCube:{value:null},tFlip:{value:-1},opacity:{value:1}},vertexShader:te.cube_vert,fragmentShader:te.cube_frag},equirect:{uniforms:{tEquirect:{value:null}},vertexShader:te.equirect_vert,fragmentShader:te.equirect_frag},distanceRGBA:{uniforms:rn([ht.common,ht.displacementmap,{referencePosition:{value:new I},nearDistance:{value:1},farDistance:{value:1e3}}]),vertexShader:te.distanceRGBA_vert,fragmentShader:te.distanceRGBA_frag},shadow:{uniforms:rn([ht.lights,ht.fog,{color:{value:new Pt(0)},opacity:{value:1}}]),vertexShader:te.shadow_vert,fragmentShader:te.shadow_frag}};Yn.physical={uniforms:rn([Yn.standard.uniforms,{clearcoat:{value:0},clearcoatMap:{value:null},clearcoatMapTransform:{value:new Kt},clearcoatNormalMap:{value:null},clearcoatNormalMapTransform:{value:new Kt},clearcoatNormalScale:{value:new j(1,1)},clearcoatRoughness:{value:0},clearcoatRoughnessMap:{value:null},clearcoatRoughnessMapTransform:{value:new Kt},dispersion:{value:0},iridescence:{value:0},iridescenceMap:{value:null},iridescenceMapTransform:{value:new Kt},iridescenceIOR:{value:1.3},iridescenceThicknessMinimum:{value:100},iridescenceThicknessMaximum:{value:400},iridescenceThicknessMap:{value:null},iridescenceThicknessMapTransform:{value:new Kt},sheen:{value:0},sheenColor:{value:new Pt(0)},sheenColorMap:{value:null},sheenColorMapTransform:{value:new Kt},sheenRoughness:{value:1},sheenRoughnessMap:{value:null},sheenRoughnessMapTransform:{value:new Kt},transmission:{value:0},transmissionMap:{value:null},transmissionMapTransform:{value:new Kt},transmissionSamplerSize:{value:new j},transmissionSamplerMap:{value:null},thickness:{value:0},thicknessMap:{value:null},thicknessMapTransform:{value:new Kt},attenuationDistance:{value:0},attenuationColor:{value:new Pt(0)},specularColor:{value:new Pt(1,1,1)},specularColorMap:{value:null},specularColorMapTransform:{value:new Kt},specularIntensity:{value:1},specularIntensityMap:{value:null},specularIntensityMapTransform:{value:new Kt},anisotropyVector:{value:new j},anisotropyMap:{value:null},anisotropyMapTransform:{value:new Kt}}]),vertexShader:te.meshphysical_vert,fragmentShader:te.meshphysical_frag};var mo={r:0,b:0,g:0},Ps=new ti,ox=new pe;function lx(n,t,e,i,s,r,a){let o=new Pt(0),l=r===!0?0:1,c,h,u=null,d=0,f=null;function g(T){let M=T.isScene===!0?T.background:null;return M&&M.isTexture&&(M=(T.backgroundBlurriness>0?e:t).get(M)),M}function x(T){let M=!1,y=g(T);y===null?p(o,l):y&&y.isColor&&(p(y,1),M=!0);let D=n.xr.getEnvironmentBlendMode();D==="additive"?i.buffers.color.setClear(0,0,0,1,a):D==="alpha-blend"&&i.buffers.color.setClear(0,0,0,0,a),(n.autoClear||M)&&(i.buffers.depth.setTest(!0),i.buffers.depth.setMask(!0),i.buffers.color.setMask(!0),n.clear(n.autoClearColor,n.autoClearDepth,n.autoClearStencil))}function m(T,M){let y=g(M);y&&(y.isCubeTexture||y.mapping===sl)?(h===void 0&&(h=new Ot(new Vn(1,1,1),new Te({name:"BackgroundCubeMaterial",uniforms:Pr(Yn.backgroundCube.uniforms),vertexShader:Yn.backgroundCube.vertexShader,fragmentShader:Yn.backgroundCube.fragmentShader,side:je,depthTest:!1,depthWrite:!1,fog:!1})),h.geometry.deleteAttribute("normal"),h.geometry.deleteAttribute("uv"),h.onBeforeRender=function(D,S,C){this.matrixWorld.copyPosition(C.matrixWorld)},Object.defineProperty(h.material,"envMap",{get:function(){return this.uniforms.envMap.value}}),s.update(h)),Ps.copy(M.backgroundRotation),Ps.x*=-1,Ps.y*=-1,Ps.z*=-1,y.isCubeTexture&&y.isRenderTargetTexture===!1&&(Ps.y*=-1,Ps.z*=-1),h.material.uniforms.envMap.value=y,h.material.uniforms.flipEnvMap.value=y.isCubeTexture&&y.isRenderTargetTexture===!1?-1:1,h.material.uniforms.backgroundBlurriness.value=M.backgroundBlurriness,h.material.uniforms.backgroundIntensity.value=M.backgroundIntensity,h.material.uniforms.backgroundRotation.value.setFromMatrix4(ox.makeRotationFromEuler(Ps)),h.material.toneMapped=se.getTransfer(y.colorSpace)!==fe,(u!==y||d!==y.version||f!==n.toneMapping)&&(h.material.needsUpdate=!0,u=y,d=y.version,f=n.toneMapping),h.layers.enableAll(),T.unshift(h,h.geometry,h.material,0,0,null)):y&&y.isTexture&&(c===void 0&&(c=new Ot(new ei(2,2),new Te({name:"BackgroundMaterial",uniforms:Pr(Yn.background.uniforms),vertexShader:Yn.background.vertexShader,fragmentShader:Yn.background.fragmentShader,side:Qi,depthTest:!1,depthWrite:!1,fog:!1})),c.geometry.deleteAttribute("normal"),Object.defineProperty(c.material,"map",{get:function(){return this.uniforms.t2D.value}}),s.update(c)),c.material.uniforms.t2D.value=y,c.material.uniforms.backgroundIntensity.value=M.backgroundIntensity,c.material.toneMapped=se.getTransfer(y.colorSpace)!==fe,y.matrixAutoUpdate===!0&&y.updateMatrix(),c.material.uniforms.uvTransform.value.copy(y.matrix),(u!==y||d!==y.version||f!==n.toneMapping)&&(c.material.needsUpdate=!0,u=y,d=y.version,f=n.toneMapping),c.layers.enableAll(),T.unshift(c,c.geometry,c.material,0,0,null))}function p(T,M){T.getRGB(mo,Mf(n)),i.buffers.color.setClear(mo.r,mo.g,mo.b,M,a)}return{getClearColor:function(){return o},setClearColor:function(T,M=1){o.set(T),l=M,p(o,l)},getClearAlpha:function(){return l},setClearAlpha:function(T){l=T,p(o,l)},render:x,addToRenderList:m}}function cx(n,t){let e=n.getParameter(n.MAX_VERTEX_ATTRIBS),i={},s=d(null),r=s,a=!1;function o(_,R,G,N,z){let J=!1,V=u(N,G,R);r!==V&&(r=V,c(r.object)),J=f(_,N,G,z),J&&g(_,N,G,z),z!==null&&t.update(z,n.ELEMENT_ARRAY_BUFFER),(J||a)&&(a=!1,y(_,R,G,N),z!==null&&n.bindBuffer(n.ELEMENT_ARRAY_BUFFER,t.get(z).buffer))}function l(){return n.createVertexArray()}function c(_){return n.bindVertexArray(_)}function h(_){return n.deleteVertexArray(_)}function u(_,R,G){let N=G.wireframe===!0,z=i[_.id];z===void 0&&(z={},i[_.id]=z);let J=z[R.id];J===void 0&&(J={},z[R.id]=J);let V=J[N];return V===void 0&&(V=d(l()),J[N]=V),V}function d(_){let R=[],G=[],N=[];for(let z=0;z<e;z++)R[z]=0,G[z]=0,N[z]=0;return{geometry:null,program:null,wireframe:!1,newAttributes:R,enabledAttributes:G,attributeDivisors:N,object:_,attributes:{},index:null}}function f(_,R,G,N){let z=r.attributes,J=R.attributes,V=0,it=G.getAttributes();for(let W in it)if(it[W].location>=0){let gt=z[W],Mt=J[W];if(Mt===void 0&&(W==="instanceMatrix"&&_.instanceMatrix&&(Mt=_.instanceMatrix),W==="instanceColor"&&_.instanceColor&&(Mt=_.instanceColor)),gt===void 0||gt.attribute!==Mt||Mt&&gt.data!==Mt.data)return!0;V++}return r.attributesNum!==V||r.index!==N}function g(_,R,G,N){let z={},J=R.attributes,V=0,it=G.getAttributes();for(let W in it)if(it[W].location>=0){let gt=J[W];gt===void 0&&(W==="instanceMatrix"&&_.instanceMatrix&&(gt=_.instanceMatrix),W==="instanceColor"&&_.instanceColor&&(gt=_.instanceColor));let Mt={};Mt.attribute=gt,gt&&gt.data&&(Mt.data=gt.data),z[W]=Mt,V++}r.attributes=z,r.attributesNum=V,r.index=N}function x(){let _=r.newAttributes;for(let R=0,G=_.length;R<G;R++)_[R]=0}function m(_){p(_,0)}function p(_,R){let G=r.newAttributes,N=r.enabledAttributes,z=r.attributeDivisors;G[_]=1,N[_]===0&&(n.enableVertexAttribArray(_),N[_]=1),z[_]!==R&&(n.vertexAttribDivisor(_,R),z[_]=R)}function T(){let _=r.newAttributes,R=r.enabledAttributes;for(let G=0,N=R.length;G<N;G++)R[G]!==_[G]&&(n.disableVertexAttribArray(G),R[G]=0)}function M(_,R,G,N,z,J,V){V===!0?n.vertexAttribIPointer(_,R,G,z,J):n.vertexAttribPointer(_,R,G,N,z,J)}function y(_,R,G,N){x();let z=N.attributes,J=G.getAttributes(),V=R.defaultAttributeValues;for(let it in J){let W=J[it];if(W.location>=0){let ct=z[it];if(ct===void 0&&(it==="instanceMatrix"&&_.instanceMatrix&&(ct=_.instanceMatrix),it==="instanceColor"&&_.instanceColor&&(ct=_.instanceColor)),ct!==void 0){let gt=ct.normalized,Mt=ct.itemSize,Yt=t.get(ct);if(Yt===void 0)continue;let he=Yt.buffer,Y=Yt.type,rt=Yt.bytesPerElement,St=Y===n.INT||Y===n.UNSIGNED_INT||ct.gpuType===eu;if(ct.isInterleavedBufferAttribute){let ot=ct.data,Bt=ot.stride,qt=ct.offset;if(ot.isInstancedInterleavedBuffer){for(let Vt=0;Vt<W.locationSize;Vt++)p(W.location+Vt,ot.meshPerAttribute);_.isInstancedMesh!==!0&&N._maxInstanceCount===void 0&&(N._maxInstanceCount=ot.meshPerAttribute*ot.count)}else for(let Vt=0;Vt<W.locationSize;Vt++)m(W.location+Vt);n.bindBuffer(n.ARRAY_BUFFER,he);for(let Vt=0;Vt<W.locationSize;Vt++)M(W.location+Vt,Mt/W.locationSize,Y,gt,Bt*rt,(qt+Mt/W.locationSize*Vt)*rt,St)}else{if(ct.isInstancedBufferAttribute){for(let ot=0;ot<W.locationSize;ot++)p(W.location+ot,ct.meshPerAttribute);_.isInstancedMesh!==!0&&N._maxInstanceCount===void 0&&(N._maxInstanceCount=ct.meshPerAttribute*ct.count)}else for(let ot=0;ot<W.locationSize;ot++)m(W.location+ot);n.bindBuffer(n.ARRAY_BUFFER,he);for(let ot=0;ot<W.locationSize;ot++)M(W.location+ot,Mt/W.locationSize,Y,gt,Mt*rt,Mt/W.locationSize*ot*rt,St)}}else if(V!==void 0){let gt=V[it];if(gt!==void 0)switch(gt.length){case 2:n.vertexAttrib2fv(W.location,gt);break;case 3:n.vertexAttrib3fv(W.location,gt);break;case 4:n.vertexAttrib4fv(W.location,gt);break;default:n.vertexAttrib1fv(W.location,gt)}}}}T()}function D(){P();for(let _ in i){let R=i[_];for(let G in R){let N=R[G];for(let z in N)h(N[z].object),delete N[z];delete R[G]}delete i[_]}}function S(_){if(i[_.id]===void 0)return;let R=i[_.id];for(let G in R){let N=R[G];for(let z in N)h(N[z].object),delete N[z];delete R[G]}delete i[_.id]}function C(_){for(let R in i){let G=i[R];if(G[_.id]===void 0)continue;let N=G[_.id];for(let z in N)h(N[z].object),delete N[z];delete G[_.id]}}function P(){w(),a=!0,r!==s&&(r=s,c(r.object))}function w(){s.geometry=null,s.program=null,s.wireframe=!1}return{setup:o,reset:P,resetDefaultState:w,dispose:D,releaseStatesOfGeometry:S,releaseStatesOfProgram:C,initAttributes:x,enableAttribute:m,disableUnusedAttributes:T}}function hx(n,t,e){let i;function s(c){i=c}function r(c,h){n.drawArrays(i,c,h),e.update(h,i,1)}function a(c,h,u){u!==0&&(n.drawArraysInstanced(i,c,h,u),e.update(h,i,u))}function o(c,h,u){if(u===0)return;t.get("WEBGL_multi_draw").multiDrawArraysWEBGL(i,c,0,h,0,u);let f=0;for(let g=0;g<u;g++)f+=h[g];e.update(f,i,1)}function l(c,h,u,d){if(u===0)return;let f=t.get("WEBGL_multi_draw");if(f===null)for(let g=0;g<c.length;g++)a(c[g],h[g],d[g]);else{f.multiDrawArraysInstancedWEBGL(i,c,0,h,0,d,0,u);let g=0;for(let x=0;x<u;x++)g+=h[x]*d[x];e.update(g,i,1)}}this.setMode=s,this.render=r,this.renderInstances=a,this.renderMultiDraw=o,this.renderMultiDrawInstances=l}function ux(n,t,e,i){let s;function r(){if(s!==void 0)return s;if(t.has("EXT_texture_filter_anisotropic")===!0){let C=t.get("EXT_texture_filter_anisotropic");s=n.getParameter(C.MAX_TEXTURE_MAX_ANISOTROPY_EXT)}else s=0;return s}function a(C){return!(C!==Hn&&i.convert(C)!==n.getParameter(n.IMPLEMENTATION_COLOR_READ_FORMAT))}function o(C){let P=C===en&&(t.has("EXT_color_buffer_half_float")||t.has("EXT_color_buffer_float"));return!(C!==Ai&&i.convert(C)!==n.getParameter(n.IMPLEMENTATION_COLOR_READ_TYPE)&&C!==Kn&&!P)}function l(C){if(C==="highp"){if(n.getShaderPrecisionFormat(n.VERTEX_SHADER,n.HIGH_FLOAT).precision>0&&n.getShaderPrecisionFormat(n.FRAGMENT_SHADER,n.HIGH_FLOAT).precision>0)return"highp";C="mediump"}return C==="mediump"&&n.getShaderPrecisionFormat(n.VERTEX_SHADER,n.MEDIUM_FLOAT).precision>0&&n.getShaderPrecisionFormat(n.FRAGMENT_SHADER,n.MEDIUM_FLOAT).precision>0?"mediump":"lowp"}let c=e.precision!==void 0?e.precision:"highp",h=l(c);h!==c&&(console.warn("THREE.WebGLRenderer:",c,"not supported, using",h,"instead."),c=h);let u=e.logarithmicDepthBuffer===!0,d=e.reverseDepthBuffer===!0&&t.has("EXT_clip_control"),f=n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS),g=n.getParameter(n.MAX_VERTEX_TEXTURE_IMAGE_UNITS),x=n.getParameter(n.MAX_TEXTURE_SIZE),m=n.getParameter(n.MAX_CUBE_MAP_TEXTURE_SIZE),p=n.getParameter(n.MAX_VERTEX_ATTRIBS),T=n.getParameter(n.MAX_VERTEX_UNIFORM_VECTORS),M=n.getParameter(n.MAX_VARYING_VECTORS),y=n.getParameter(n.MAX_FRAGMENT_UNIFORM_VECTORS),D=g>0,S=n.getParameter(n.MAX_SAMPLES);return{isWebGL2:!0,getMaxAnisotropy:r,getMaxPrecision:l,textureFormatReadable:a,textureTypeReadable:o,precision:c,logarithmicDepthBuffer:u,reverseDepthBuffer:d,maxTextures:f,maxVertexTextures:g,maxTextureSize:x,maxCubemapSize:m,maxAttributes:p,maxVertexUniforms:T,maxVaryings:M,maxFragmentUniforms:y,vertexTextures:D,maxSamples:S}}function dx(n){let t=this,e=null,i=0,s=!1,r=!1,a=new Mi,o=new Kt,l={value:null,needsUpdate:!1};this.uniform=l,this.numPlanes=0,this.numIntersection=0,this.init=function(u,d){let f=u.length!==0||d||i!==0||s;return s=d,i=u.length,f},this.beginShadows=function(){r=!0,h(null)},this.endShadows=function(){r=!1},this.setGlobalState=function(u,d){e=h(u,d,0)},this.setState=function(u,d,f){let g=u.clippingPlanes,x=u.clipIntersection,m=u.clipShadows,p=n.get(u);if(!s||g===null||g.length===0||r&&!m)r?h(null):c();else{let T=r?0:i,M=T*4,y=p.clippingState||null;l.value=y,y=h(g,d,M,f);for(let D=0;D!==M;++D)y[D]=e[D];p.clippingState=y,this.numIntersection=x?this.numPlanes:0,this.numPlanes+=T}};function c(){l.value!==e&&(l.value=e,l.needsUpdate=i>0),t.numPlanes=i,t.numIntersection=0}function h(u,d,f,g){let x=u!==null?u.length:0,m=null;if(x!==0){if(m=l.value,g!==!0||m===null){let p=f+x*4,T=d.matrixWorldInverse;o.getNormalMatrix(T),(m===null||m.length<p)&&(m=new Float32Array(p));for(let M=0,y=f;M!==x;++M,y+=4)a.copy(u[M]).applyMatrix4(T,o),a.normal.toArray(m,y),m[y+3]=a.constant}l.value=m,l.needsUpdate=!0}return t.numPlanes=x,t.numIntersection=0,m}}function fx(n){let t=new WeakMap;function e(a,o){return o===Rc?a.mapping=Tr:o===Pc&&(a.mapping=Ar),a}function i(a){if(a&&a.isTexture){let o=a.mapping;if(o===Rc||o===Pc)if(t.has(a)){let l=t.get(a).texture;return e(l,a.mapping)}else{let l=a.image;if(l&&l.height>0){let c=new fh(l.height);return c.fromEquirectangularTexture(n,a),t.set(a,c),a.addEventListener("dispose",s),e(c.texture,a.mapping)}else return null}}return a}function s(a){let o=a.target;o.removeEventListener("dispose",s);let l=t.get(o);l!==void 0&&(t.delete(o),l.dispose())}function r(){t=new WeakMap}return{get:i,dispose:r}}var Ir=class extends Bo{constructor(t=-1,e=1,i=1,s=-1,r=.1,a=2e3){super(),this.isOrthographicCamera=!0,this.type="OrthographicCamera",this.zoom=1,this.view=null,this.left=t,this.right=e,this.top=i,this.bottom=s,this.near=r,this.far=a,this.updateProjectionMatrix()}copy(t,e){return super.copy(t,e),this.left=t.left,this.right=t.right,this.top=t.top,this.bottom=t.bottom,this.near=t.near,this.far=t.far,this.zoom=t.zoom,this.view=t.view===null?null:Object.assign({},t.view),this}setViewOffset(t,e,i,s,r,a){this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=t,this.view.fullHeight=e,this.view.offsetX=i,this.view.offsetY=s,this.view.width=r,this.view.height=a,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){let t=(this.right-this.left)/(2*this.zoom),e=(this.top-this.bottom)/(2*this.zoom),i=(this.right+this.left)/2,s=(this.top+this.bottom)/2,r=i-t,a=i+t,o=s+e,l=s-e;if(this.view!==null&&this.view.enabled){let c=(this.right-this.left)/this.view.fullWidth/this.zoom,h=(this.top-this.bottom)/this.view.fullHeight/this.zoom;r+=c*this.view.offsetX,a=r+c*this.view.width,o-=h*this.view.offsetY,l=o-h*this.view.height}this.projectionMatrix.makeOrthographic(r,a,o,l,this.near,this.far,this.coordinateSystem),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(t){let e=super.toJSON(t);return e.object.zoom=this.zoom,e.object.left=this.left,e.object.right=this.right,e.object.top=this.top,e.object.bottom=this.bottom,e.object.near=this.near,e.object.far=this.far,this.view!==null&&(e.object.view=Object.assign({},this.view)),e}},yr=4,Md=[.125,.215,.35,.446,.526,.582],Ds=20,cc=new Ir,Sd=new Pt,hc=null,uc=0,dc=0,fc=!1,Fs=(1+Math.sqrt(5))/2,fr=1/Fs,wd=[new I(-Fs,fr,0),new I(Fs,fr,0),new I(-fr,0,Fs),new I(fr,0,Fs),new I(0,Fs,-fr),new I(0,Fs,fr),new I(-1,1,-1),new I(1,1,-1),new I(-1,1,1),new I(1,1,1)],Fr=class{constructor(t){this._renderer=t,this._pingPongRenderTarget=null,this._lodMax=0,this._cubeSize=0,this._lodPlanes=[],this._sizeLods=[],this._sigmas=[],this._blurMaterial=null,this._cubemapMaterial=null,this._equirectMaterial=null,this._compileMaterial(this._blurMaterial)}fromScene(t,e=0,i=.1,s=100){hc=this._renderer.getRenderTarget(),uc=this._renderer.getActiveCubeFace(),dc=this._renderer.getActiveMipmapLevel(),fc=this._renderer.xr.enabled,this._renderer.xr.enabled=!1,this._setSize(256);let r=this._allocateTargets();return r.depthBuffer=!0,this._sceneToCubeUV(t,i,s,r),e>0&&this._blur(r,0,0,e),this._applyPMREM(r),this._cleanup(r),r}fromEquirectangular(t,e=null){return this._fromTexture(t,e)}fromCubemap(t,e=null){return this._fromTexture(t,e)}compileCubemapShader(){this._cubemapMaterial===null&&(this._cubemapMaterial=Ad(),this._compileMaterial(this._cubemapMaterial))}compileEquirectangularShader(){this._equirectMaterial===null&&(this._equirectMaterial=Td(),this._compileMaterial(this._equirectMaterial))}dispose(){this._dispose(),this._cubemapMaterial!==null&&this._cubemapMaterial.dispose(),this._equirectMaterial!==null&&this._equirectMaterial.dispose()}_setSize(t){this._lodMax=Math.floor(Math.log2(t)),this._cubeSize=Math.pow(2,this._lodMax)}_dispose(){this._blurMaterial!==null&&this._blurMaterial.dispose(),this._pingPongRenderTarget!==null&&this._pingPongRenderTarget.dispose();for(let t=0;t<this._lodPlanes.length;t++)this._lodPlanes[t].dispose()}_cleanup(t){this._renderer.setRenderTarget(hc,uc,dc),this._renderer.xr.enabled=fc,t.scissorTest=!1,go(t,0,0,t.width,t.height)}_fromTexture(t,e){t.mapping===Tr||t.mapping===Ar?this._setSize(t.image.length===0?16:t.image[0].width||t.image[0].image.width):this._setSize(t.image.width/4),hc=this._renderer.getRenderTarget(),uc=this._renderer.getActiveCubeFace(),dc=this._renderer.getActiveMipmapLevel(),fc=this._renderer.xr.enabled,this._renderer.xr.enabled=!1;let i=e||this._allocateTargets();return this._textureToCubeUV(t,i),this._applyPMREM(i),this._cleanup(i),i}_allocateTargets(){let t=3*Math.max(this._cubeSize,112),e=4*this._cubeSize,i={magFilter:Jn,minFilter:Jn,generateMipmaps:!1,type:en,format:Hn,colorSpace:Lr,depthBuffer:!1},s=Ed(t,e,i);if(this._pingPongRenderTarget===null||this._pingPongRenderTarget.width!==t||this._pingPongRenderTarget.height!==e){this._pingPongRenderTarget!==null&&this._dispose(),this._pingPongRenderTarget=Ed(t,e,i);let{_lodMax:r}=this;({sizeLods:this._sizeLods,lodPlanes:this._lodPlanes,sigmas:this._sigmas}=px(r)),this._blurMaterial=mx(r,t,e)}return s}_compileMaterial(t){let e=new Ot(this._lodPlanes[0],t);this._renderer.compile(e,cc)}_sceneToCubeUV(t,e,i,s){let o=new Qe(90,1,e,i),l=[1,-1,1,1,1,1],c=[1,1,1,-1,-1,-1],h=this._renderer,u=h.autoClear,d=h.toneMapping;h.getClearColor(Sd),h.toneMapping=Ji,h.autoClear=!1;let f=new Ve({name:"PMREM.Background",side:je,depthWrite:!1,depthTest:!1}),g=new Ot(new Vn,f),x=!1,m=t.background;m?m.isColor&&(f.color.copy(m),t.background=null,x=!0):(f.color.copy(Sd),x=!0);for(let p=0;p<6;p++){let T=p%3;T===0?(o.up.set(0,l[p],0),o.lookAt(c[p],0,0)):T===1?(o.up.set(0,0,l[p]),o.lookAt(0,c[p],0)):(o.up.set(0,l[p],0),o.lookAt(0,0,c[p]));let M=this._cubeSize;go(s,T*M,p>2?M:0,M,M),h.setRenderTarget(s),x&&h.render(g,o),h.render(t,o)}g.geometry.dispose(),g.material.dispose(),h.toneMapping=d,h.autoClear=u,t.background=m}_textureToCubeUV(t,e){let i=this._renderer,s=t.mapping===Tr||t.mapping===Ar;s?(this._cubemapMaterial===null&&(this._cubemapMaterial=Ad()),this._cubemapMaterial.uniforms.flipEnvMap.value=t.isRenderTargetTexture===!1?-1:1):this._equirectMaterial===null&&(this._equirectMaterial=Td());let r=s?this._cubemapMaterial:this._equirectMaterial,a=new Ot(this._lodPlanes[0],r),o=r.uniforms;o.envMap.value=t;let l=this._cubeSize;go(e,0,0,3*l,2*l),i.setRenderTarget(e),i.render(a,cc)}_applyPMREM(t){let e=this._renderer,i=e.autoClear;e.autoClear=!1;let s=this._lodPlanes.length;for(let r=1;r<s;r++){let a=Math.sqrt(this._sigmas[r]*this._sigmas[r]-this._sigmas[r-1]*this._sigmas[r-1]),o=wd[(s-r-1)%wd.length];this._blur(t,r-1,r,a,o)}e.autoClear=i}_blur(t,e,i,s,r){let a=this._pingPongRenderTarget;this._halfBlur(t,a,e,i,s,"latitudinal",r),this._halfBlur(a,t,i,i,s,"longitudinal",r)}_halfBlur(t,e,i,s,r,a,o){let l=this._renderer,c=this._blurMaterial;a!=="latitudinal"&&a!=="longitudinal"&&console.error("blur direction must be either latitudinal or longitudinal!");let h=3,u=new Ot(this._lodPlanes[s],c),d=c.uniforms,f=this._sizeLods[i]-1,g=isFinite(r)?Math.PI/(2*f):2*Math.PI/(2*Ds-1),x=r/g,m=isFinite(r)?1+Math.floor(h*x):Ds;m>Ds&&console.warn(`sigmaRadians, ${r}, is too large and will clip, as it requested ${m} samples when the maximum is set to ${Ds}`);let p=[],T=0;for(let C=0;C<Ds;++C){let P=C/x,w=Math.exp(-P*P/2);p.push(w),C===0?T+=w:C<m&&(T+=2*w)}for(let C=0;C<p.length;C++)p[C]=p[C]/T;d.envMap.value=t.texture,d.samples.value=m,d.weights.value=p,d.latitudinal.value=a==="latitudinal",o&&(d.poleAxis.value=o);let{_lodMax:M}=this;d.dTheta.value=g,d.mipInt.value=M-i;let y=this._sizeLods[s],D=3*y*(s>M-yr?s-M+yr:0),S=4*(this._cubeSize-y);go(e,D,S,3*y,2*y),l.setRenderTarget(e),l.render(u,cc)}};function px(n){let t=[],e=[],i=[],s=n,r=n-yr+1+Md.length;for(let a=0;a<r;a++){let o=Math.pow(2,s);e.push(o);let l=1/o;a>n-yr?l=Md[a-n+yr-1]:a===0&&(l=0),i.push(l);let c=1/(o-2),h=-c,u=1+c,d=[h,h,u,h,u,u,h,h,u,u,h,u],f=6,g=6,x=3,m=2,p=1,T=new Float32Array(x*g*f),M=new Float32Array(m*g*f),y=new Float32Array(p*g*f);for(let S=0;S<f;S++){let C=S%3*2/3-1,P=S>2?0:-1,w=[C,P,0,C+2/3,P,0,C+2/3,P+1,0,C,P,0,C+2/3,P+1,0,C,P+1,0];T.set(w,x*g*S),M.set(d,m*g*S);let _=[S,S,S,S,S,S];y.set(_,p*g*S)}let D=new tn;D.setAttribute("position",new an(T,x)),D.setAttribute("uv",new an(M,m)),D.setAttribute("faceIndex",new an(y,p)),t.push(D),s>yr&&s--}return{lodPlanes:t,sizeLods:e,sigmas:i}}function Ed(n,t,e){let i=new Ue(n,t,e);return i.texture.mapping=sl,i.texture.name="PMREM.cubeUv",i.scissorTest=!0,i}function go(n,t,e,i,s){n.viewport.set(t,e,i,s),n.scissor.set(t,e,i,s)}function mx(n,t,e){let i=new Float32Array(Ds),s=new I(0,1,0);return new Te({name:"SphericalGaussianBlur",defines:{n:Ds,CUBEUV_TEXEL_WIDTH:1/t,CUBEUV_TEXEL_HEIGHT:1/e,CUBEUV_MAX_MIP:`${n}.0`},uniforms:{envMap:{value:null},samples:{value:1},weights:{value:i},latitudinal:{value:!1},dTheta:{value:0},mipInt:{value:0},poleAxis:{value:s}},vertexShader:lu(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;
			uniform int samples;
			uniform float weights[ n ];
			uniform bool latitudinal;
			uniform float dTheta;
			uniform float mipInt;
			uniform vec3 poleAxis;

			#define ENVMAP_TYPE_CUBE_UV
			#include <cube_uv_reflection_fragment>

			vec3 getSample( float theta, vec3 axis ) {

				float cosTheta = cos( theta );
				// Rodrigues' axis-angle rotation
				vec3 sampleDirection = vOutputDirection * cosTheta
					+ cross( axis, vOutputDirection ) * sin( theta )
					+ axis * dot( axis, vOutputDirection ) * ( 1.0 - cosTheta );

				return bilinearCubeUV( envMap, sampleDirection, mipInt );

			}

			void main() {

				vec3 axis = latitudinal ? poleAxis : cross( poleAxis, vOutputDirection );

				if ( all( equal( axis, vec3( 0.0 ) ) ) ) {

					axis = vec3( vOutputDirection.z, 0.0, - vOutputDirection.x );

				}

				axis = normalize( axis );

				gl_FragColor = vec4( 0.0, 0.0, 0.0, 1.0 );
				gl_FragColor.rgb += weights[ 0 ] * getSample( 0.0, axis );

				for ( int i = 1; i < n; i++ ) {

					if ( i >= samples ) {

						break;

					}

					float theta = dTheta * float( i );
					gl_FragColor.rgb += weights[ i ] * getSample( -1.0 * theta, axis );
					gl_FragColor.rgb += weights[ i ] * getSample( theta, axis );

				}

			}
		`,blending:Rn,depthTest:!1,depthWrite:!1})}function Td(){return new Te({name:"EquirectangularToCubeUV",uniforms:{envMap:{value:null}},vertexShader:lu(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;

			#include <common>

			void main() {

				vec3 outputDirection = normalize( vOutputDirection );
				vec2 uv = equirectUv( outputDirection );

				gl_FragColor = vec4( texture2D ( envMap, uv ).rgb, 1.0 );

			}
		`,blending:Rn,depthTest:!1,depthWrite:!1})}function Ad(){return new Te({name:"CubemapToCubeUV",uniforms:{envMap:{value:null},flipEnvMap:{value:-1}},vertexShader:lu(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			uniform float flipEnvMap;

			varying vec3 vOutputDirection;

			uniform samplerCube envMap;

			void main() {

				gl_FragColor = textureCube( envMap, vec3( flipEnvMap * vOutputDirection.x, vOutputDirection.yz ) );

			}
		`,blending:Rn,depthTest:!1,depthWrite:!1})}function lu(){return`

		precision mediump float;
		precision mediump int;

		attribute float faceIndex;

		varying vec3 vOutputDirection;

		// RH coordinate system; PMREM face-indexing convention
		vec3 getDirection( vec2 uv, float face ) {

			uv = 2.0 * uv - 1.0;

			vec3 direction = vec3( uv, 1.0 );

			if ( face == 0.0 ) {

				direction = direction.zyx; // ( 1, v, u ) pos x

			} else if ( face == 1.0 ) {

				direction = direction.xzy;
				direction.xz *= -1.0; // ( -u, 1, -v ) pos y

			} else if ( face == 2.0 ) {

				direction.x *= -1.0; // ( -u, v, 1 ) pos z

			} else if ( face == 3.0 ) {

				direction = direction.zyx;
				direction.xz *= -1.0; // ( -1, v, -u ) neg x

			} else if ( face == 4.0 ) {

				direction = direction.xzy;
				direction.xy *= -1.0; // ( -u, -1, v ) neg y

			} else if ( face == 5.0 ) {

				direction.z *= -1.0; // ( u, v, -1 ) neg z

			}

			return direction;

		}

		void main() {

			vOutputDirection = getDirection( uv, faceIndex );
			gl_Position = vec4( position, 1.0 );

		}
	`}function gx(n){let t=new WeakMap,e=null;function i(o){if(o&&o.isTexture){let l=o.mapping,c=l===Rc||l===Pc,h=l===Tr||l===Ar;if(c||h){let u=t.get(o),d=u!==void 0?u.texture.pmremVersion:0;if(o.isRenderTargetTexture&&o.pmremVersion!==d)return e===null&&(e=new Fr(n)),u=c?e.fromEquirectangular(o,u):e.fromCubemap(o,u),u.texture.pmremVersion=o.pmremVersion,t.set(o,u),u.texture;if(u!==void 0)return u.texture;{let f=o.image;return c&&f&&f.height>0||h&&f&&s(f)?(e===null&&(e=new Fr(n)),u=c?e.fromEquirectangular(o):e.fromCubemap(o),u.texture.pmremVersion=o.pmremVersion,t.set(o,u),o.addEventListener("dispose",r),u.texture):null}}}return o}function s(o){let l=0,c=6;for(let h=0;h<c;h++)o[h]!==void 0&&l++;return l===c}function r(o){let l=o.target;l.removeEventListener("dispose",r);let c=t.get(l);c!==void 0&&(t.delete(l),c.dispose())}function a(){t=new WeakMap,e!==null&&(e.dispose(),e=null)}return{get:i,dispose:a}}function xx(n){let t={};function e(i){if(t[i]!==void 0)return t[i];let s;switch(i){case"WEBGL_depth_texture":s=n.getExtension("WEBGL_depth_texture")||n.getExtension("MOZ_WEBGL_depth_texture")||n.getExtension("WEBKIT_WEBGL_depth_texture");break;case"EXT_texture_filter_anisotropic":s=n.getExtension("EXT_texture_filter_anisotropic")||n.getExtension("MOZ_EXT_texture_filter_anisotropic")||n.getExtension("WEBKIT_EXT_texture_filter_anisotropic");break;case"WEBGL_compressed_texture_s3tc":s=n.getExtension("WEBGL_compressed_texture_s3tc")||n.getExtension("MOZ_WEBGL_compressed_texture_s3tc")||n.getExtension("WEBKIT_WEBGL_compressed_texture_s3tc");break;case"WEBGL_compressed_texture_pvrtc":s=n.getExtension("WEBGL_compressed_texture_pvrtc")||n.getExtension("WEBKIT_WEBGL_compressed_texture_pvrtc");break;default:s=n.getExtension(i)}return t[i]=s,s}return{has:function(i){return e(i)!==null},init:function(){e("EXT_color_buffer_float"),e("WEBGL_clip_cull_distance"),e("OES_texture_float_linear"),e("EXT_color_buffer_half_float"),e("WEBGL_multisampled_render_to_texture"),e("WEBGL_render_shared_exponent")},get:function(i){let s=e(i);return s===null&&aa("THREE.WebGLRenderer: "+i+" extension not supported."),s}}}function vx(n,t,e,i){let s={},r=new WeakMap;function a(u){let d=u.target;d.index!==null&&t.remove(d.index);for(let g in d.attributes)t.remove(d.attributes[g]);for(let g in d.morphAttributes){let x=d.morphAttributes[g];for(let m=0,p=x.length;m<p;m++)t.remove(x[m])}d.removeEventListener("dispose",a),delete s[d.id];let f=r.get(d);f&&(t.remove(f),r.delete(d)),i.releaseStatesOfGeometry(d),d.isInstancedBufferGeometry===!0&&delete d._maxInstanceCount,e.memory.geometries--}function o(u,d){return s[d.id]===!0||(d.addEventListener("dispose",a),s[d.id]=!0,e.memory.geometries++),d}function l(u){let d=u.attributes;for(let g in d)t.update(d[g],n.ARRAY_BUFFER);let f=u.morphAttributes;for(let g in f){let x=f[g];for(let m=0,p=x.length;m<p;m++)t.update(x[m],n.ARRAY_BUFFER)}}function c(u){let d=[],f=u.index,g=u.attributes.position,x=0;if(f!==null){let T=f.array;x=f.version;for(let M=0,y=T.length;M<y;M+=3){let D=T[M+0],S=T[M+1],C=T[M+2];d.push(D,S,S,C,C,D)}}else if(g!==void 0){let T=g.array;x=g.version;for(let M=0,y=T.length/3-1;M<y;M+=3){let D=M+0,S=M+1,C=M+2;d.push(D,S,S,C,C,D)}}else return;let m=new(_f(d)?No:Uo)(d,1);m.version=x;let p=r.get(u);p&&t.remove(p),r.set(u,m)}function h(u){let d=r.get(u);if(d){let f=u.index;f!==null&&d.version<f.version&&c(u)}else c(u);return r.get(u)}return{get:o,update:l,getWireframeAttribute:h}}function yx(n,t,e){let i;function s(d){i=d}let r,a;function o(d){r=d.type,a=d.bytesPerElement}function l(d,f){n.drawElements(i,f,r,d*a),e.update(f,i,1)}function c(d,f,g){g!==0&&(n.drawElementsInstanced(i,f,r,d*a,g),e.update(f,i,g))}function h(d,f,g){if(g===0)return;t.get("WEBGL_multi_draw").multiDrawElementsWEBGL(i,f,0,r,d,0,g);let m=0;for(let p=0;p<g;p++)m+=f[p];e.update(m,i,1)}function u(d,f,g,x){if(g===0)return;let m=t.get("WEBGL_multi_draw");if(m===null)for(let p=0;p<d.length;p++)c(d[p]/a,f[p],x[p]);else{m.multiDrawElementsInstancedWEBGL(i,f,0,r,d,0,x,0,g);let p=0;for(let T=0;T<g;T++)p+=f[T]*x[T];e.update(p,i,1)}}this.setMode=s,this.setIndex=o,this.render=l,this.renderInstances=c,this.renderMultiDraw=h,this.renderMultiDrawInstances=u}function _x(n){let t={geometries:0,textures:0},e={frame:0,calls:0,triangles:0,points:0,lines:0};function i(r,a,o){switch(e.calls++,a){case n.TRIANGLES:e.triangles+=o*(r/3);break;case n.LINES:e.lines+=o*(r/2);break;case n.LINE_STRIP:e.lines+=o*(r-1);break;case n.LINE_LOOP:e.lines+=o*r;break;case n.POINTS:e.points+=o*r;break;default:console.error("THREE.WebGLInfo: Unknown draw mode:",a);break}}function s(){e.calls=0,e.triangles=0,e.points=0,e.lines=0}return{memory:t,render:e,programs:null,autoReset:!0,reset:s,update:i}}function bx(n,t,e){let i=new WeakMap,s=new xe;function r(a,o,l){let c=a.morphTargetInfluences,h=o.morphAttributes.position||o.morphAttributes.normal||o.morphAttributes.color,u=h!==void 0?h.length:0,d=i.get(o);if(d===void 0||d.count!==u){let w=function(){C.dispose(),i.delete(o),o.removeEventListener("dispose",w)};d!==void 0&&d.texture.dispose();let f=o.morphAttributes.position!==void 0,g=o.morphAttributes.normal!==void 0,x=o.morphAttributes.color!==void 0,m=o.morphAttributes.position||[],p=o.morphAttributes.normal||[],T=o.morphAttributes.color||[],M=0;f===!0&&(M=1),g===!0&&(M=2),x===!0&&(M=3);let y=o.attributes.position.count*M,D=1;y>t.maxTextureSize&&(D=Math.ceil(y/t.maxTextureSize),y=t.maxTextureSize);let S=new Float32Array(y*D*4*u),C=new ko(S,y,D,u);C.type=Kn,C.needsUpdate=!0;let P=M*4;for(let _=0;_<u;_++){let R=m[_],G=p[_],N=T[_],z=y*D*4*_;for(let J=0;J<R.count;J++){let V=J*P;f===!0&&(s.fromBufferAttribute(R,J),S[z+V+0]=s.x,S[z+V+1]=s.y,S[z+V+2]=s.z,S[z+V+3]=0),g===!0&&(s.fromBufferAttribute(G,J),S[z+V+4]=s.x,S[z+V+5]=s.y,S[z+V+6]=s.z,S[z+V+7]=0),x===!0&&(s.fromBufferAttribute(N,J),S[z+V+8]=s.x,S[z+V+9]=s.y,S[z+V+10]=s.z,S[z+V+11]=N.itemSize===4?s.w:1)}}d={count:u,texture:C,size:new j(y,D)},i.set(o,d),o.addEventListener("dispose",w)}if(a.isInstancedMesh===!0&&a.morphTexture!==null)l.getUniforms().setValue(n,"morphTexture",a.morphTexture,e);else{let f=0;for(let x=0;x<c.length;x++)f+=c[x];let g=o.morphTargetsRelative?1:1-f;l.getUniforms().setValue(n,"morphTargetBaseInfluence",g),l.getUniforms().setValue(n,"morphTargetInfluences",c)}l.getUniforms().setValue(n,"morphTargetsTexture",d.texture,e),l.getUniforms().setValue(n,"morphTargetsTextureSize",d.size)}return{update:r}}function Mx(n,t,e,i){let s=new WeakMap;function r(l){let c=i.render.frame,h=l.geometry,u=t.get(l,h);if(s.get(u)!==c&&(t.update(u),s.set(u,c)),l.isInstancedMesh&&(l.hasEventListener("dispose",o)===!1&&l.addEventListener("dispose",o),s.get(l)!==c&&(e.update(l.instanceMatrix,n.ARRAY_BUFFER),l.instanceColor!==null&&e.update(l.instanceColor,n.ARRAY_BUFFER),s.set(l,c))),l.isSkinnedMesh){let d=l.skeleton;s.get(d)!==c&&(d.update(),s.set(d,c))}return u}function a(){s=new WeakMap}function o(l){let c=l.target;c.removeEventListener("dispose",o),e.remove(c.instanceMatrix),c.instanceColor!==null&&e.remove(c.instanceColor)}return{update:r,dispose:a}}var zo=class extends on{constructor(t,e,i,s,r,a,o,l,c,h=Mr){if(h!==Mr&&h!==Rr)throw new Error("DepthTexture format must be either THREE.DepthFormat or THREE.DepthStencilFormat");i===void 0&&h===Mr&&(i=Us),i===void 0&&h===Rr&&(i=Cr),super(null,s,r,a,o,l,h,i,c),this.isDepthTexture=!0,this.image={width:t,height:e},this.magFilter=o!==void 0?o:yn,this.minFilter=l!==void 0?l:yn,this.flipY=!1,this.generateMipmaps=!1,this.compareFunction=null}copy(t){return super.copy(t),this.compareFunction=t.compareFunction,this}toJSON(t){let e=super.toJSON(t);return this.compareFunction!==null&&(e.compareFunction=this.compareFunction),e}},wf=new on,Cd=new zo(1,1),Ef=new ko,Tf=new hh,Af=new Oo,Rd=[],Pd=[],Id=new Float32Array(16),Fd=new Float32Array(9),Dd=new Float32Array(4);function Ur(n,t,e){let i=n[0];if(i<=0||i>0)return n;let s=t*e,r=Rd[s];if(r===void 0&&(r=new Float32Array(s),Rd[s]=r),t!==0){i.toArray(r,0);for(let a=1,o=0;a!==t;++a)o+=e,n[a].toArray(r,o)}return r}function Be(n,t){if(n.length!==t.length)return!1;for(let e=0,i=n.length;e<i;e++)if(n[e]!==t[e])return!1;return!0}function Oe(n,t){for(let e=0,i=t.length;e<i;e++)n[e]=t[e]}function al(n,t){let e=Pd[t];e===void 0&&(e=new Int32Array(t),Pd[t]=e);for(let i=0;i!==t;++i)e[i]=n.allocateTextureUnit();return e}function Sx(n,t){let e=this.cache;e[0]!==t&&(n.uniform1f(this.addr,t),e[0]=t)}function wx(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(n.uniform2f(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Be(e,t))return;n.uniform2fv(this.addr,t),Oe(e,t)}}function Ex(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(n.uniform3f(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else if(t.r!==void 0)(e[0]!==t.r||e[1]!==t.g||e[2]!==t.b)&&(n.uniform3f(this.addr,t.r,t.g,t.b),e[0]=t.r,e[1]=t.g,e[2]=t.b);else{if(Be(e,t))return;n.uniform3fv(this.addr,t),Oe(e,t)}}function Tx(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(n.uniform4f(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Be(e,t))return;n.uniform4fv(this.addr,t),Oe(e,t)}}function Ax(n,t){let e=this.cache,i=t.elements;if(i===void 0){if(Be(e,t))return;n.uniformMatrix2fv(this.addr,!1,t),Oe(e,t)}else{if(Be(e,i))return;Dd.set(i),n.uniformMatrix2fv(this.addr,!1,Dd),Oe(e,i)}}function Cx(n,t){let e=this.cache,i=t.elements;if(i===void 0){if(Be(e,t))return;n.uniformMatrix3fv(this.addr,!1,t),Oe(e,t)}else{if(Be(e,i))return;Fd.set(i),n.uniformMatrix3fv(this.addr,!1,Fd),Oe(e,i)}}function Rx(n,t){let e=this.cache,i=t.elements;if(i===void 0){if(Be(e,t))return;n.uniformMatrix4fv(this.addr,!1,t),Oe(e,t)}else{if(Be(e,i))return;Id.set(i),n.uniformMatrix4fv(this.addr,!1,Id),Oe(e,i)}}function Px(n,t){let e=this.cache;e[0]!==t&&(n.uniform1i(this.addr,t),e[0]=t)}function Ix(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(n.uniform2i(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Be(e,t))return;n.uniform2iv(this.addr,t),Oe(e,t)}}function Fx(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(n.uniform3i(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else{if(Be(e,t))return;n.uniform3iv(this.addr,t),Oe(e,t)}}function Dx(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(n.uniform4i(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Be(e,t))return;n.uniform4iv(this.addr,t),Oe(e,t)}}function kx(n,t){let e=this.cache;e[0]!==t&&(n.uniform1ui(this.addr,t),e[0]=t)}function Lx(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(n.uniform2ui(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Be(e,t))return;n.uniform2uiv(this.addr,t),Oe(e,t)}}function Ux(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(n.uniform3ui(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else{if(Be(e,t))return;n.uniform3uiv(this.addr,t),Oe(e,t)}}function Nx(n,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(n.uniform4ui(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Be(e,t))return;n.uniform4uiv(this.addr,t),Oe(e,t)}}function Bx(n,t,e){let i=this.cache,s=e.allocateTextureUnit();i[0]!==s&&(n.uniform1i(this.addr,s),i[0]=s);let r;this.type===n.SAMPLER_2D_SHADOW?(Cd.compareFunction=yf,r=Cd):r=wf,e.setTexture2D(t||r,s)}function Ox(n,t,e){let i=this.cache,s=e.allocateTextureUnit();i[0]!==s&&(n.uniform1i(this.addr,s),i[0]=s),e.setTexture3D(t||Tf,s)}function zx(n,t,e){let i=this.cache,s=e.allocateTextureUnit();i[0]!==s&&(n.uniform1i(this.addr,s),i[0]=s),e.setTextureCube(t||Af,s)}function Hx(n,t,e){let i=this.cache,s=e.allocateTextureUnit();i[0]!==s&&(n.uniform1i(this.addr,s),i[0]=s),e.setTexture2DArray(t||Ef,s)}function Vx(n){switch(n){case 5126:return Sx;case 35664:return wx;case 35665:return Ex;case 35666:return Tx;case 35674:return Ax;case 35675:return Cx;case 35676:return Rx;case 5124:case 35670:return Px;case 35667:case 35671:return Ix;case 35668:case 35672:return Fx;case 35669:case 35673:return Dx;case 5125:return kx;case 36294:return Lx;case 36295:return Ux;case 36296:return Nx;case 35678:case 36198:case 36298:case 36306:case 35682:return Bx;case 35679:case 36299:case 36307:return Ox;case 35680:case 36300:case 36308:case 36293:return zx;case 36289:case 36303:case 36311:case 36292:return Hx}}function Gx(n,t){n.uniform1fv(this.addr,t)}function Wx(n,t){let e=Ur(t,this.size,2);n.uniform2fv(this.addr,e)}function qx(n,t){let e=Ur(t,this.size,3);n.uniform3fv(this.addr,e)}function Xx(n,t){let e=Ur(t,this.size,4);n.uniform4fv(this.addr,e)}function $x(n,t){let e=Ur(t,this.size,4);n.uniformMatrix2fv(this.addr,!1,e)}function Yx(n,t){let e=Ur(t,this.size,9);n.uniformMatrix3fv(this.addr,!1,e)}function Zx(n,t){let e=Ur(t,this.size,16);n.uniformMatrix4fv(this.addr,!1,e)}function Jx(n,t){n.uniform1iv(this.addr,t)}function Kx(n,t){n.uniform2iv(this.addr,t)}function Qx(n,t){n.uniform3iv(this.addr,t)}function jx(n,t){n.uniform4iv(this.addr,t)}function t1(n,t){n.uniform1uiv(this.addr,t)}function e1(n,t){n.uniform2uiv(this.addr,t)}function n1(n,t){n.uniform3uiv(this.addr,t)}function i1(n,t){n.uniform4uiv(this.addr,t)}function s1(n,t,e){let i=this.cache,s=t.length,r=al(e,s);Be(i,r)||(n.uniform1iv(this.addr,r),Oe(i,r));for(let a=0;a!==s;++a)e.setTexture2D(t[a]||wf,r[a])}function r1(n,t,e){let i=this.cache,s=t.length,r=al(e,s);Be(i,r)||(n.uniform1iv(this.addr,r),Oe(i,r));for(let a=0;a!==s;++a)e.setTexture3D(t[a]||Tf,r[a])}function a1(n,t,e){let i=this.cache,s=t.length,r=al(e,s);Be(i,r)||(n.uniform1iv(this.addr,r),Oe(i,r));for(let a=0;a!==s;++a)e.setTextureCube(t[a]||Af,r[a])}function o1(n,t,e){let i=this.cache,s=t.length,r=al(e,s);Be(i,r)||(n.uniform1iv(this.addr,r),Oe(i,r));for(let a=0;a!==s;++a)e.setTexture2DArray(t[a]||Ef,r[a])}function l1(n){switch(n){case 5126:return Gx;case 35664:return Wx;case 35665:return qx;case 35666:return Xx;case 35674:return $x;case 35675:return Yx;case 35676:return Zx;case 5124:case 35670:return Jx;case 35667:case 35671:return Kx;case 35668:case 35672:return Qx;case 35669:case 35673:return jx;case 5125:return t1;case 36294:return e1;case 36295:return n1;case 36296:return i1;case 35678:case 36198:case 36298:case 36306:case 35682:return s1;case 35679:case 36299:case 36307:return r1;case 35680:case 36300:case 36308:case 36293:return a1;case 36289:case 36303:case 36311:case 36292:return o1}}var ph=class{constructor(t,e,i){this.id=t,this.addr=i,this.cache=[],this.type=e.type,this.setValue=Vx(e.type)}},mh=class{constructor(t,e,i){this.id=t,this.addr=i,this.cache=[],this.type=e.type,this.size=e.size,this.setValue=l1(e.type)}},gh=class{constructor(t){this.id=t,this.seq=[],this.map={}}setValue(t,e,i){let s=this.seq;for(let r=0,a=s.length;r!==a;++r){let o=s[r];o.setValue(t,e[o.id],i)}}},pc=/(\w+)(\])?(\[|\.)?/g;function kd(n,t){n.seq.push(t),n.map[t.id]=t}function c1(n,t,e){let i=n.name,s=i.length;for(pc.lastIndex=0;;){let r=pc.exec(i),a=pc.lastIndex,o=r[1],l=r[2]==="]",c=r[3];if(l&&(o=o|0),c===void 0||c==="["&&a+2===s){kd(e,c===void 0?new ph(o,n,t):new mh(o,n,t));break}else{let u=e.map[o];u===void 0&&(u=new gh(o),kd(e,u)),e=u}}}var wr=class{constructor(t,e){this.seq=[],this.map={};let i=t.getProgramParameter(e,t.ACTIVE_UNIFORMS);for(let s=0;s<i;++s){let r=t.getActiveUniform(e,s),a=t.getUniformLocation(e,r.name);c1(r,a,this)}}setValue(t,e,i,s){let r=this.map[e];r!==void 0&&r.setValue(t,i,s)}setOptional(t,e,i){let s=e[i];s!==void 0&&this.setValue(t,i,s)}static upload(t,e,i,s){for(let r=0,a=e.length;r!==a;++r){let o=e[r],l=i[o.id];l.needsUpdate!==!1&&o.setValue(t,l.value,s)}}static seqWithValue(t,e){let i=[];for(let s=0,r=t.length;s!==r;++s){let a=t[s];a.id in e&&i.push(a)}return i}};function Ld(n,t,e){let i=n.createShader(t);return n.shaderSource(i,e),n.compileShader(i),i}var h1=37297,u1=0;function d1(n,t){let e=n.split(`
`),i=[],s=Math.max(t-6,0),r=Math.min(t+6,e.length);for(let a=s;a<r;a++){let o=a+1;i.push(`${o===t?">":" "} ${o}: ${e[a]}`)}return i.join(`
`)}var Ud=new Kt;function f1(n){se._getMatrix(Ud,se.workingColorSpace,n);let t=`mat3( ${Ud.elements.map(e=>e.toFixed(4))} )`;switch(se.getTransfer(n)){case rl:return[t,"LinearTransferOETF"];case fe:return[t,"sRGBTransferOETF"];default:return console.warn("THREE.WebGLProgram: Unsupported color space: ",n),[t,"LinearTransferOETF"]}}function Nd(n,t,e){let i=n.getShaderParameter(t,n.COMPILE_STATUS),s=n.getShaderInfoLog(t).trim();if(i&&s==="")return"";let r=/ERROR: 0:(\d+)/.exec(s);if(r){let a=parseInt(r[1]);return e.toUpperCase()+`

`+s+`

`+d1(n.getShaderSource(t),a)}else return s}function p1(n,t){let e=f1(t);return[`vec4 ${n}( vec4 value ) {`,`	return ${e[1]}( vec4( value.rgb * ${e[0]}, value.a ) );`,"}"].join(`
`)}function m1(n,t){let e;switch(t){case Jh:e="Linear";break;case Kh:e="Reinhard";break;case Qh:e="Cineon";break;case Ta:e="ACESFilmic";break;case jh:e="AgX";break;case tu:e="Neutral";break;case $p:e="Custom";break;default:console.warn("THREE.WebGLProgram: Unsupported toneMapping:",t),e="Linear"}return"vec3 "+n+"( vec3 color ) { return "+e+"ToneMapping( color ); }"}var xo=new I;function g1(){se.getLuminanceCoefficients(xo);let n=xo.x.toFixed(4),t=xo.y.toFixed(4),e=xo.z.toFixed(4);return["float luminance( const in vec3 rgb ) {",`	const vec3 weights = vec3( ${n}, ${t}, ${e} );`,"	return dot( weights, rgb );","}"].join(`
`)}function x1(n){return[n.extensionClipCullDistance?"#extension GL_ANGLE_clip_cull_distance : require":"",n.extensionMultiDraw?"#extension GL_ANGLE_multi_draw : require":""].filter(oa).join(`
`)}function v1(n){let t=[];for(let e in n){let i=n[e];i!==!1&&t.push("#define "+e+" "+i)}return t.join(`
`)}function y1(n,t){let e={},i=n.getProgramParameter(t,n.ACTIVE_ATTRIBUTES);for(let s=0;s<i;s++){let r=n.getActiveAttrib(t,s),a=r.name,o=1;r.type===n.FLOAT_MAT2&&(o=2),r.type===n.FLOAT_MAT3&&(o=3),r.type===n.FLOAT_MAT4&&(o=4),e[a]={type:r.type,location:n.getAttribLocation(t,a),locationSize:o}}return e}function oa(n){return n!==""}function Bd(n,t){let e=t.numSpotLightShadows+t.numSpotLightMaps-t.numSpotLightShadowsWithMaps;return n.replace(/NUM_DIR_LIGHTS/g,t.numDirLights).replace(/NUM_SPOT_LIGHTS/g,t.numSpotLights).replace(/NUM_SPOT_LIGHT_MAPS/g,t.numSpotLightMaps).replace(/NUM_SPOT_LIGHT_COORDS/g,e).replace(/NUM_RECT_AREA_LIGHTS/g,t.numRectAreaLights).replace(/NUM_POINT_LIGHTS/g,t.numPointLights).replace(/NUM_HEMI_LIGHTS/g,t.numHemiLights).replace(/NUM_DIR_LIGHT_SHADOWS/g,t.numDirLightShadows).replace(/NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS/g,t.numSpotLightShadowsWithMaps).replace(/NUM_SPOT_LIGHT_SHADOWS/g,t.numSpotLightShadows).replace(/NUM_POINT_LIGHT_SHADOWS/g,t.numPointLightShadows)}function Od(n,t){return n.replace(/NUM_CLIPPING_PLANES/g,t.numClippingPlanes).replace(/UNION_CLIPPING_PLANES/g,t.numClippingPlanes-t.numClipIntersection)}var _1=/^[ \t]*#include +<([\w\d./]+)>/gm;function xh(n){return n.replace(_1,M1)}var b1=new Map;function M1(n,t){let e=te[t];if(e===void 0){let i=b1.get(t);if(i!==void 0)e=te[i],console.warn('THREE.WebGLRenderer: Shader chunk "%s" has been deprecated. Use "%s" instead.',t,i);else throw new Error("Can not resolve #include <"+t+">")}return xh(e)}var S1=/#pragma unroll_loop_start\s+for\s*\(\s*int\s+i\s*=\s*(\d+)\s*;\s*i\s*<\s*(\d+)\s*;\s*i\s*\+\+\s*\)\s*{([\s\S]+?)}\s+#pragma unroll_loop_end/g;function zd(n){return n.replace(S1,w1)}function w1(n,t,e,i){let s="";for(let r=parseInt(t);r<parseInt(e);r++)s+=i.replace(/\[\s*i\s*\]/g,"[ "+r+" ]").replace(/UNROLLED_LOOP_INDEX/g,r);return s}function Hd(n){let t=`precision ${n.precision} float;
	precision ${n.precision} int;
	precision ${n.precision} sampler2D;
	precision ${n.precision} samplerCube;
	precision ${n.precision} sampler3D;
	precision ${n.precision} sampler2DArray;
	precision ${n.precision} sampler2DShadow;
	precision ${n.precision} samplerCubeShadow;
	precision ${n.precision} sampler2DArrayShadow;
	precision ${n.precision} isampler2D;
	precision ${n.precision} isampler3D;
	precision ${n.precision} isamplerCube;
	precision ${n.precision} isampler2DArray;
	precision ${n.precision} usampler2D;
	precision ${n.precision} usampler3D;
	precision ${n.precision} usamplerCube;
	precision ${n.precision} usampler2DArray;
	`;return n.precision==="highp"?t+=`
#define HIGH_PRECISION`:n.precision==="mediump"?t+=`
#define MEDIUM_PRECISION`:n.precision==="lowp"&&(t+=`
#define LOW_PRECISION`),t}function E1(n){let t="SHADOWMAP_TYPE_BASIC";return n.shadowMapType===af?t="SHADOWMAP_TYPE_PCF":n.shadowMapType===Rp?t="SHADOWMAP_TYPE_PCF_SOFT":n.shadowMapType===bi&&(t="SHADOWMAP_TYPE_VSM"),t}function T1(n){let t="ENVMAP_TYPE_CUBE";if(n.envMap)switch(n.envMapMode){case Tr:case Ar:t="ENVMAP_TYPE_CUBE";break;case sl:t="ENVMAP_TYPE_CUBE_UV";break}return t}function A1(n){let t="ENVMAP_MODE_REFLECTION";if(n.envMap)switch(n.envMapMode){case Ar:t="ENVMAP_MODE_REFRACTION";break}return t}function C1(n){let t="ENVMAP_BLENDING_NONE";if(n.envMap)switch(n.combine){case of:t="ENVMAP_BLENDING_MULTIPLY";break;case qp:t="ENVMAP_BLENDING_MIX";break;case Xp:t="ENVMAP_BLENDING_ADD";break}return t}function R1(n){let t=n.envMapCubeUVHeight;if(t===null)return null;let e=Math.log2(t)-2,i=1/t;return{texelWidth:1/(3*Math.max(Math.pow(2,e),112)),texelHeight:i,maxMip:e}}function P1(n,t,e,i){let s=n.getContext(),r=e.defines,a=e.vertexShader,o=e.fragmentShader,l=E1(e),c=T1(e),h=A1(e),u=C1(e),d=R1(e),f=x1(e),g=v1(r),x=s.createProgram(),m,p,T=e.glslVersion?"#version "+e.glslVersion+`
`:"";e.isRawShaderMaterial?(m=["#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g].filter(oa).join(`
`),m.length>0&&(m+=`
`),p=["#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g].filter(oa).join(`
`),p.length>0&&(p+=`
`)):(m=[Hd(e),"#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g,e.extensionClipCullDistance?"#define USE_CLIP_DISTANCE":"",e.batching?"#define USE_BATCHING":"",e.batchingColor?"#define USE_BATCHING_COLOR":"",e.instancing?"#define USE_INSTANCING":"",e.instancingColor?"#define USE_INSTANCING_COLOR":"",e.instancingMorph?"#define USE_INSTANCING_MORPH":"",e.useFog&&e.fog?"#define USE_FOG":"",e.useFog&&e.fogExp2?"#define FOG_EXP2":"",e.map?"#define USE_MAP":"",e.envMap?"#define USE_ENVMAP":"",e.envMap?"#define "+h:"",e.lightMap?"#define USE_LIGHTMAP":"",e.aoMap?"#define USE_AOMAP":"",e.bumpMap?"#define USE_BUMPMAP":"",e.normalMap?"#define USE_NORMALMAP":"",e.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",e.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",e.displacementMap?"#define USE_DISPLACEMENTMAP":"",e.emissiveMap?"#define USE_EMISSIVEMAP":"",e.anisotropy?"#define USE_ANISOTROPY":"",e.anisotropyMap?"#define USE_ANISOTROPYMAP":"",e.clearcoatMap?"#define USE_CLEARCOATMAP":"",e.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",e.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",e.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",e.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",e.specularMap?"#define USE_SPECULARMAP":"",e.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",e.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",e.roughnessMap?"#define USE_ROUGHNESSMAP":"",e.metalnessMap?"#define USE_METALNESSMAP":"",e.alphaMap?"#define USE_ALPHAMAP":"",e.alphaHash?"#define USE_ALPHAHASH":"",e.transmission?"#define USE_TRANSMISSION":"",e.transmissionMap?"#define USE_TRANSMISSIONMAP":"",e.thicknessMap?"#define USE_THICKNESSMAP":"",e.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",e.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",e.mapUv?"#define MAP_UV "+e.mapUv:"",e.alphaMapUv?"#define ALPHAMAP_UV "+e.alphaMapUv:"",e.lightMapUv?"#define LIGHTMAP_UV "+e.lightMapUv:"",e.aoMapUv?"#define AOMAP_UV "+e.aoMapUv:"",e.emissiveMapUv?"#define EMISSIVEMAP_UV "+e.emissiveMapUv:"",e.bumpMapUv?"#define BUMPMAP_UV "+e.bumpMapUv:"",e.normalMapUv?"#define NORMALMAP_UV "+e.normalMapUv:"",e.displacementMapUv?"#define DISPLACEMENTMAP_UV "+e.displacementMapUv:"",e.metalnessMapUv?"#define METALNESSMAP_UV "+e.metalnessMapUv:"",e.roughnessMapUv?"#define ROUGHNESSMAP_UV "+e.roughnessMapUv:"",e.anisotropyMapUv?"#define ANISOTROPYMAP_UV "+e.anisotropyMapUv:"",e.clearcoatMapUv?"#define CLEARCOATMAP_UV "+e.clearcoatMapUv:"",e.clearcoatNormalMapUv?"#define CLEARCOAT_NORMALMAP_UV "+e.clearcoatNormalMapUv:"",e.clearcoatRoughnessMapUv?"#define CLEARCOAT_ROUGHNESSMAP_UV "+e.clearcoatRoughnessMapUv:"",e.iridescenceMapUv?"#define IRIDESCENCEMAP_UV "+e.iridescenceMapUv:"",e.iridescenceThicknessMapUv?"#define IRIDESCENCE_THICKNESSMAP_UV "+e.iridescenceThicknessMapUv:"",e.sheenColorMapUv?"#define SHEEN_COLORMAP_UV "+e.sheenColorMapUv:"",e.sheenRoughnessMapUv?"#define SHEEN_ROUGHNESSMAP_UV "+e.sheenRoughnessMapUv:"",e.specularMapUv?"#define SPECULARMAP_UV "+e.specularMapUv:"",e.specularColorMapUv?"#define SPECULAR_COLORMAP_UV "+e.specularColorMapUv:"",e.specularIntensityMapUv?"#define SPECULAR_INTENSITYMAP_UV "+e.specularIntensityMapUv:"",e.transmissionMapUv?"#define TRANSMISSIONMAP_UV "+e.transmissionMapUv:"",e.thicknessMapUv?"#define THICKNESSMAP_UV "+e.thicknessMapUv:"",e.vertexTangents&&e.flatShading===!1?"#define USE_TANGENT":"",e.vertexColors?"#define USE_COLOR":"",e.vertexAlphas?"#define USE_COLOR_ALPHA":"",e.vertexUv1s?"#define USE_UV1":"",e.vertexUv2s?"#define USE_UV2":"",e.vertexUv3s?"#define USE_UV3":"",e.pointsUvs?"#define USE_POINTS_UV":"",e.flatShading?"#define FLAT_SHADED":"",e.skinning?"#define USE_SKINNING":"",e.morphTargets?"#define USE_MORPHTARGETS":"",e.morphNormals&&e.flatShading===!1?"#define USE_MORPHNORMALS":"",e.morphColors?"#define USE_MORPHCOLORS":"",e.morphTargetsCount>0?"#define MORPHTARGETS_TEXTURE_STRIDE "+e.morphTextureStride:"",e.morphTargetsCount>0?"#define MORPHTARGETS_COUNT "+e.morphTargetsCount:"",e.doubleSided?"#define DOUBLE_SIDED":"",e.flipSided?"#define FLIP_SIDED":"",e.shadowMapEnabled?"#define USE_SHADOWMAP":"",e.shadowMapEnabled?"#define "+l:"",e.sizeAttenuation?"#define USE_SIZEATTENUATION":"",e.numLightProbes>0?"#define USE_LIGHT_PROBES":"",e.logarithmicDepthBuffer?"#define USE_LOGDEPTHBUF":"",e.reverseDepthBuffer?"#define USE_REVERSEDEPTHBUF":"","uniform mat4 modelMatrix;","uniform mat4 modelViewMatrix;","uniform mat4 projectionMatrix;","uniform mat4 viewMatrix;","uniform mat3 normalMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;","#ifdef USE_INSTANCING","	attribute mat4 instanceMatrix;","#endif","#ifdef USE_INSTANCING_COLOR","	attribute vec3 instanceColor;","#endif","#ifdef USE_INSTANCING_MORPH","	uniform sampler2D morphTexture;","#endif","attribute vec3 position;","attribute vec3 normal;","attribute vec2 uv;","#ifdef USE_UV1","	attribute vec2 uv1;","#endif","#ifdef USE_UV2","	attribute vec2 uv2;","#endif","#ifdef USE_UV3","	attribute vec2 uv3;","#endif","#ifdef USE_TANGENT","	attribute vec4 tangent;","#endif","#if defined( USE_COLOR_ALPHA )","	attribute vec4 color;","#elif defined( USE_COLOR )","	attribute vec3 color;","#endif","#ifdef USE_SKINNING","	attribute vec4 skinIndex;","	attribute vec4 skinWeight;","#endif",`
`].filter(oa).join(`
`),p=[Hd(e),"#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,g,e.useFog&&e.fog?"#define USE_FOG":"",e.useFog&&e.fogExp2?"#define FOG_EXP2":"",e.alphaToCoverage?"#define ALPHA_TO_COVERAGE":"",e.map?"#define USE_MAP":"",e.matcap?"#define USE_MATCAP":"",e.envMap?"#define USE_ENVMAP":"",e.envMap?"#define "+c:"",e.envMap?"#define "+h:"",e.envMap?"#define "+u:"",d?"#define CUBEUV_TEXEL_WIDTH "+d.texelWidth:"",d?"#define CUBEUV_TEXEL_HEIGHT "+d.texelHeight:"",d?"#define CUBEUV_MAX_MIP "+d.maxMip+".0":"",e.lightMap?"#define USE_LIGHTMAP":"",e.aoMap?"#define USE_AOMAP":"",e.bumpMap?"#define USE_BUMPMAP":"",e.normalMap?"#define USE_NORMALMAP":"",e.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",e.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",e.emissiveMap?"#define USE_EMISSIVEMAP":"",e.anisotropy?"#define USE_ANISOTROPY":"",e.anisotropyMap?"#define USE_ANISOTROPYMAP":"",e.clearcoat?"#define USE_CLEARCOAT":"",e.clearcoatMap?"#define USE_CLEARCOATMAP":"",e.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",e.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",e.dispersion?"#define USE_DISPERSION":"",e.iridescence?"#define USE_IRIDESCENCE":"",e.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",e.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",e.specularMap?"#define USE_SPECULARMAP":"",e.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",e.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",e.roughnessMap?"#define USE_ROUGHNESSMAP":"",e.metalnessMap?"#define USE_METALNESSMAP":"",e.alphaMap?"#define USE_ALPHAMAP":"",e.alphaTest?"#define USE_ALPHATEST":"",e.alphaHash?"#define USE_ALPHAHASH":"",e.sheen?"#define USE_SHEEN":"",e.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",e.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",e.transmission?"#define USE_TRANSMISSION":"",e.transmissionMap?"#define USE_TRANSMISSIONMAP":"",e.thicknessMap?"#define USE_THICKNESSMAP":"",e.vertexTangents&&e.flatShading===!1?"#define USE_TANGENT":"",e.vertexColors||e.instancingColor||e.batchingColor?"#define USE_COLOR":"",e.vertexAlphas?"#define USE_COLOR_ALPHA":"",e.vertexUv1s?"#define USE_UV1":"",e.vertexUv2s?"#define USE_UV2":"",e.vertexUv3s?"#define USE_UV3":"",e.pointsUvs?"#define USE_POINTS_UV":"",e.gradientMap?"#define USE_GRADIENTMAP":"",e.flatShading?"#define FLAT_SHADED":"",e.doubleSided?"#define DOUBLE_SIDED":"",e.flipSided?"#define FLIP_SIDED":"",e.shadowMapEnabled?"#define USE_SHADOWMAP":"",e.shadowMapEnabled?"#define "+l:"",e.premultipliedAlpha?"#define PREMULTIPLIED_ALPHA":"",e.numLightProbes>0?"#define USE_LIGHT_PROBES":"",e.decodeVideoTexture?"#define DECODE_VIDEO_TEXTURE":"",e.decodeVideoTextureEmissive?"#define DECODE_VIDEO_TEXTURE_EMISSIVE":"",e.logarithmicDepthBuffer?"#define USE_LOGDEPTHBUF":"",e.reverseDepthBuffer?"#define USE_REVERSEDEPTHBUF":"","uniform mat4 viewMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;",e.toneMapping!==Ji?"#define TONE_MAPPING":"",e.toneMapping!==Ji?te.tonemapping_pars_fragment:"",e.toneMapping!==Ji?m1("toneMapping",e.toneMapping):"",e.dithering?"#define DITHERING":"",e.opaque?"#define OPAQUE":"",te.colorspace_pars_fragment,p1("linearToOutputTexel",e.outputColorSpace),g1(),e.useDepthPacking?"#define DEPTH_PACKING "+e.depthPacking:"",`
`].filter(oa).join(`
`)),a=xh(a),a=Bd(a,e),a=Od(a,e),o=xh(o),o=Bd(o,e),o=Od(o,e),a=zd(a),o=zd(o),e.isRawShaderMaterial!==!0&&(T=`#version 300 es
`,m=[f,"#define attribute in","#define varying out","#define texture2D texture"].join(`
`)+`
`+m,p=["#define varying in",e.glslVersion===ed?"":"layout(location = 0) out highp vec4 pc_fragColor;",e.glslVersion===ed?"":"#define gl_FragColor pc_fragColor","#define gl_FragDepthEXT gl_FragDepth","#define texture2D texture","#define textureCube texture","#define texture2DProj textureProj","#define texture2DLodEXT textureLod","#define texture2DProjLodEXT textureProjLod","#define textureCubeLodEXT textureLod","#define texture2DGradEXT textureGrad","#define texture2DProjGradEXT textureProjGrad","#define textureCubeGradEXT textureGrad"].join(`
`)+`
`+p);let M=T+m+a,y=T+p+o,D=Ld(s,s.VERTEX_SHADER,M),S=Ld(s,s.FRAGMENT_SHADER,y);s.attachShader(x,D),s.attachShader(x,S),e.index0AttributeName!==void 0?s.bindAttribLocation(x,0,e.index0AttributeName):e.morphTargets===!0&&s.bindAttribLocation(x,0,"position"),s.linkProgram(x);function C(R){if(n.debug.checkShaderErrors){let G=s.getProgramInfoLog(x).trim(),N=s.getShaderInfoLog(D).trim(),z=s.getShaderInfoLog(S).trim(),J=!0,V=!0;if(s.getProgramParameter(x,s.LINK_STATUS)===!1)if(J=!1,typeof n.debug.onShaderError=="function")n.debug.onShaderError(s,x,D,S);else{let it=Nd(s,D,"vertex"),W=Nd(s,S,"fragment");console.error("THREE.WebGLProgram: Shader Error "+s.getError()+" - VALIDATE_STATUS "+s.getProgramParameter(x,s.VALIDATE_STATUS)+`

Material Name: `+R.name+`
Material Type: `+R.type+`

Program Info Log: `+G+`
`+it+`
`+W)}else G!==""?console.warn("THREE.WebGLProgram: Program Info Log:",G):(N===""||z==="")&&(V=!1);V&&(R.diagnostics={runnable:J,programLog:G,vertexShader:{log:N,prefix:m},fragmentShader:{log:z,prefix:p}})}s.deleteShader(D),s.deleteShader(S),P=new wr(s,x),w=y1(s,x)}let P;this.getUniforms=function(){return P===void 0&&C(this),P};let w;this.getAttributes=function(){return w===void 0&&C(this),w};let _=e.rendererExtensionParallelShaderCompile===!1;return this.isReady=function(){return _===!1&&(_=s.getProgramParameter(x,h1)),_},this.destroy=function(){i.releaseStatesOfProgram(this),s.deleteProgram(x),this.program=void 0},this.type=e.shaderType,this.name=e.shaderName,this.id=u1++,this.cacheKey=t,this.usedTimes=1,this.program=x,this.vertexShader=D,this.fragmentShader=S,this}var I1=0,vh=class{constructor(){this.shaderCache=new Map,this.materialCache=new Map}update(t){let e=t.vertexShader,i=t.fragmentShader,s=this._getShaderStage(e),r=this._getShaderStage(i),a=this._getShaderCacheForMaterial(t);return a.has(s)===!1&&(a.add(s),s.usedTimes++),a.has(r)===!1&&(a.add(r),r.usedTimes++),this}remove(t){let e=this.materialCache.get(t);for(let i of e)i.usedTimes--,i.usedTimes===0&&this.shaderCache.delete(i.code);return this.materialCache.delete(t),this}getVertexShaderID(t){return this._getShaderStage(t.vertexShader).id}getFragmentShaderID(t){return this._getShaderStage(t.fragmentShader).id}dispose(){this.shaderCache.clear(),this.materialCache.clear()}_getShaderCacheForMaterial(t){let e=this.materialCache,i=e.get(t);return i===void 0&&(i=new Set,e.set(t,i)),i}_getShaderStage(t){let e=this.shaderCache,i=e.get(t);return i===void 0&&(i=new yh(t),e.set(t,i)),i}},yh=class{constructor(t){this.id=I1++,this.code=t,this.usedTimes=0}};function F1(n,t,e,i,s,r,a){let o=new Lo,l=new vh,c=new Set,h=[],u=s.logarithmicDepthBuffer,d=s.vertexTextures,f=s.precision,g={MeshDepthMaterial:"depth",MeshDistanceMaterial:"distanceRGBA",MeshNormalMaterial:"normal",MeshBasicMaterial:"basic",MeshLambertMaterial:"lambert",MeshPhongMaterial:"phong",MeshToonMaterial:"toon",MeshStandardMaterial:"physical",MeshPhysicalMaterial:"physical",MeshMatcapMaterial:"matcap",LineBasicMaterial:"basic",LineDashedMaterial:"dashed",PointsMaterial:"points",ShadowMaterial:"shadow",SpriteMaterial:"sprite"};function x(w){return c.add(w),w===0?"uv":`uv${w}`}function m(w,_,R,G,N){let z=G.fog,J=N.geometry,V=w.isMeshStandardMaterial?G.environment:null,it=(w.isMeshStandardMaterial?e:t).get(w.envMap||V),W=it&&it.mapping===sl?it.image.height:null,ct=g[w.type];w.precision!==null&&(f=s.getMaxPrecision(w.precision),f!==w.precision&&console.warn("THREE.WebGLProgram.getParameters:",w.precision,"not supported, using",f,"instead."));let gt=J.morphAttributes.position||J.morphAttributes.normal||J.morphAttributes.color,Mt=gt!==void 0?gt.length:0,Yt=0;J.morphAttributes.position!==void 0&&(Yt=1),J.morphAttributes.normal!==void 0&&(Yt=2),J.morphAttributes.color!==void 0&&(Yt=3);let he,Y,rt,St;if(ct){let me=Yn[ct];he=me.vertexShader,Y=me.fragmentShader}else he=w.vertexShader,Y=w.fragmentShader,l.update(w),rt=l.getVertexShaderID(w),St=l.getFragmentShaderID(w);let ot=n.getRenderTarget(),Bt=n.state.buffers.depth.getReversed(),qt=N.isInstancedMesh===!0,Vt=N.isBatchedMesh===!0,le=!!w.map,Q=!!w.matcap,st=!!it,A=!!w.aoMap,Ft=!!w.lightMap,et=!!w.bumpMap,yt=!!w.normalMap,lt=!!w.displacementMap,zt=!!w.emissiveMap,xt=!!w.metalnessMap,E=!!w.roughnessMap,v=w.anisotropy>0,U=w.clearcoat>0,X=w.dispersion>0,tt=w.iridescence>0,$=w.sheen>0,Et=w.transmission>0,ut=v&&!!w.anisotropyMap,vt=U&&!!w.clearcoatMap,ne=U&&!!w.clearcoatNormalMap,nt=U&&!!w.clearcoatRoughnessMap,_t=tt&&!!w.iridescenceMap,Ht=tt&&!!w.iridescenceThicknessMap,Wt=$&&!!w.sheenColorMap,bt=$&&!!w.sheenRoughnessMap,ae=!!w.specularMap,jt=!!w.specularColorMap,ve=!!w.specularIntensityMap,F=Et&&!!w.transmissionMap,dt=Et&&!!w.thicknessMap,q=!!w.gradientMap,K=!!w.alphaMap,mt=w.alphaTest>0,ft=!!w.alphaHash,Zt=!!w.extensions,Ie=Ji;w.toneMapped&&(ot===null||ot.isXRRenderTarget===!0)&&(Ie=n.toneMapping);let Ye={shaderID:ct,shaderType:w.type,shaderName:w.name,vertexShader:he,fragmentShader:Y,defines:w.defines,customVertexShaderID:rt,customFragmentShaderID:St,isRawShaderMaterial:w.isRawShaderMaterial===!0,glslVersion:w.glslVersion,precision:f,batching:Vt,batchingColor:Vt&&N._colorsTexture!==null,instancing:qt,instancingColor:qt&&N.instanceColor!==null,instancingMorph:qt&&N.morphTexture!==null,supportsVertexTextures:d,outputColorSpace:ot===null?n.outputColorSpace:ot.isXRRenderTarget===!0?ot.texture.colorSpace:Lr,alphaToCoverage:!!w.alphaToCoverage,map:le,matcap:Q,envMap:st,envMapMode:st&&it.mapping,envMapCubeUVHeight:W,aoMap:A,lightMap:Ft,bumpMap:et,normalMap:yt,displacementMap:d&&lt,emissiveMap:zt,normalMapObjectSpace:yt&&w.normalMapType===Kp,normalMapTangentSpace:yt&&w.normalMapType===vf,metalnessMap:xt,roughnessMap:E,anisotropy:v,anisotropyMap:ut,clearcoat:U,clearcoatMap:vt,clearcoatNormalMap:ne,clearcoatRoughnessMap:nt,dispersion:X,iridescence:tt,iridescenceMap:_t,iridescenceThicknessMap:Ht,sheen:$,sheenColorMap:Wt,sheenRoughnessMap:bt,specularMap:ae,specularColorMap:jt,specularIntensityMap:ve,transmission:Et,transmissionMap:F,thicknessMap:dt,gradientMap:q,opaque:w.transparent===!1&&w.blending===br&&w.alphaToCoverage===!1,alphaMap:K,alphaTest:mt,alphaHash:ft,combine:w.combine,mapUv:le&&x(w.map.channel),aoMapUv:A&&x(w.aoMap.channel),lightMapUv:Ft&&x(w.lightMap.channel),bumpMapUv:et&&x(w.bumpMap.channel),normalMapUv:yt&&x(w.normalMap.channel),displacementMapUv:lt&&x(w.displacementMap.channel),emissiveMapUv:zt&&x(w.emissiveMap.channel),metalnessMapUv:xt&&x(w.metalnessMap.channel),roughnessMapUv:E&&x(w.roughnessMap.channel),anisotropyMapUv:ut&&x(w.anisotropyMap.channel),clearcoatMapUv:vt&&x(w.clearcoatMap.channel),clearcoatNormalMapUv:ne&&x(w.clearcoatNormalMap.channel),clearcoatRoughnessMapUv:nt&&x(w.clearcoatRoughnessMap.channel),iridescenceMapUv:_t&&x(w.iridescenceMap.channel),iridescenceThicknessMapUv:Ht&&x(w.iridescenceThicknessMap.channel),sheenColorMapUv:Wt&&x(w.sheenColorMap.channel),sheenRoughnessMapUv:bt&&x(w.sheenRoughnessMap.channel),specularMapUv:ae&&x(w.specularMap.channel),specularColorMapUv:jt&&x(w.specularColorMap.channel),specularIntensityMapUv:ve&&x(w.specularIntensityMap.channel),transmissionMapUv:F&&x(w.transmissionMap.channel),thicknessMapUv:dt&&x(w.thicknessMap.channel),alphaMapUv:K&&x(w.alphaMap.channel),vertexTangents:!!J.attributes.tangent&&(yt||v),vertexColors:w.vertexColors,vertexAlphas:w.vertexColors===!0&&!!J.attributes.color&&J.attributes.color.itemSize===4,pointsUvs:N.isPoints===!0&&!!J.attributes.uv&&(le||K),fog:!!z,useFog:w.fog===!0,fogExp2:!!z&&z.isFogExp2,flatShading:w.flatShading===!0,sizeAttenuation:w.sizeAttenuation===!0,logarithmicDepthBuffer:u,reverseDepthBuffer:Bt,skinning:N.isSkinnedMesh===!0,morphTargets:J.morphAttributes.position!==void 0,morphNormals:J.morphAttributes.normal!==void 0,morphColors:J.morphAttributes.color!==void 0,morphTargetsCount:Mt,morphTextureStride:Yt,numDirLights:_.directional.length,numPointLights:_.point.length,numSpotLights:_.spot.length,numSpotLightMaps:_.spotLightMap.length,numRectAreaLights:_.rectArea.length,numHemiLights:_.hemi.length,numDirLightShadows:_.directionalShadowMap.length,numPointLightShadows:_.pointShadowMap.length,numSpotLightShadows:_.spotShadowMap.length,numSpotLightShadowsWithMaps:_.numSpotLightShadowsWithMaps,numLightProbes:_.numLightProbes,numClippingPlanes:a.numPlanes,numClipIntersection:a.numIntersection,dithering:w.dithering,shadowMapEnabled:n.shadowMap.enabled&&R.length>0,shadowMapType:n.shadowMap.type,toneMapping:Ie,decodeVideoTexture:le&&w.map.isVideoTexture===!0&&se.getTransfer(w.map.colorSpace)===fe,decodeVideoTextureEmissive:zt&&w.emissiveMap.isVideoTexture===!0&&se.getTransfer(w.emissiveMap.colorSpace)===fe,premultipliedAlpha:w.premultipliedAlpha,doubleSided:w.side===vn,flipSided:w.side===je,useDepthPacking:w.depthPacking>=0,depthPacking:w.depthPacking||0,index0AttributeName:w.index0AttributeName,extensionClipCullDistance:Zt&&w.extensions.clipCullDistance===!0&&i.has("WEBGL_clip_cull_distance"),extensionMultiDraw:(Zt&&w.extensions.multiDraw===!0||Vt)&&i.has("WEBGL_multi_draw"),rendererExtensionParallelShaderCompile:i.has("KHR_parallel_shader_compile"),customProgramCacheKey:w.customProgramCacheKey()};return Ye.vertexUv1s=c.has(1),Ye.vertexUv2s=c.has(2),Ye.vertexUv3s=c.has(3),c.clear(),Ye}function p(w){let _=[];if(w.shaderID?_.push(w.shaderID):(_.push(w.customVertexShaderID),_.push(w.customFragmentShaderID)),w.defines!==void 0)for(let R in w.defines)_.push(R),_.push(w.defines[R]);return w.isRawShaderMaterial===!1&&(T(_,w),M(_,w),_.push(n.outputColorSpace)),_.push(w.customProgramCacheKey),_.join()}function T(w,_){w.push(_.precision),w.push(_.outputColorSpace),w.push(_.envMapMode),w.push(_.envMapCubeUVHeight),w.push(_.mapUv),w.push(_.alphaMapUv),w.push(_.lightMapUv),w.push(_.aoMapUv),w.push(_.bumpMapUv),w.push(_.normalMapUv),w.push(_.displacementMapUv),w.push(_.emissiveMapUv),w.push(_.metalnessMapUv),w.push(_.roughnessMapUv),w.push(_.anisotropyMapUv),w.push(_.clearcoatMapUv),w.push(_.clearcoatNormalMapUv),w.push(_.clearcoatRoughnessMapUv),w.push(_.iridescenceMapUv),w.push(_.iridescenceThicknessMapUv),w.push(_.sheenColorMapUv),w.push(_.sheenRoughnessMapUv),w.push(_.specularMapUv),w.push(_.specularColorMapUv),w.push(_.specularIntensityMapUv),w.push(_.transmissionMapUv),w.push(_.thicknessMapUv),w.push(_.combine),w.push(_.fogExp2),w.push(_.sizeAttenuation),w.push(_.morphTargetsCount),w.push(_.morphAttributeCount),w.push(_.numDirLights),w.push(_.numPointLights),w.push(_.numSpotLights),w.push(_.numSpotLightMaps),w.push(_.numHemiLights),w.push(_.numRectAreaLights),w.push(_.numDirLightShadows),w.push(_.numPointLightShadows),w.push(_.numSpotLightShadows),w.push(_.numSpotLightShadowsWithMaps),w.push(_.numLightProbes),w.push(_.shadowMapType),w.push(_.toneMapping),w.push(_.numClippingPlanes),w.push(_.numClipIntersection),w.push(_.depthPacking)}function M(w,_){o.disableAll(),_.supportsVertexTextures&&o.enable(0),_.instancing&&o.enable(1),_.instancingColor&&o.enable(2),_.instancingMorph&&o.enable(3),_.matcap&&o.enable(4),_.envMap&&o.enable(5),_.normalMapObjectSpace&&o.enable(6),_.normalMapTangentSpace&&o.enable(7),_.clearcoat&&o.enable(8),_.iridescence&&o.enable(9),_.alphaTest&&o.enable(10),_.vertexColors&&o.enable(11),_.vertexAlphas&&o.enable(12),_.vertexUv1s&&o.enable(13),_.vertexUv2s&&o.enable(14),_.vertexUv3s&&o.enable(15),_.vertexTangents&&o.enable(16),_.anisotropy&&o.enable(17),_.alphaHash&&o.enable(18),_.batching&&o.enable(19),_.dispersion&&o.enable(20),_.batchingColor&&o.enable(21),w.push(o.mask),o.disableAll(),_.fog&&o.enable(0),_.useFog&&o.enable(1),_.flatShading&&o.enable(2),_.logarithmicDepthBuffer&&o.enable(3),_.reverseDepthBuffer&&o.enable(4),_.skinning&&o.enable(5),_.morphTargets&&o.enable(6),_.morphNormals&&o.enable(7),_.morphColors&&o.enable(8),_.premultipliedAlpha&&o.enable(9),_.shadowMapEnabled&&o.enable(10),_.doubleSided&&o.enable(11),_.flipSided&&o.enable(12),_.useDepthPacking&&o.enable(13),_.dithering&&o.enable(14),_.transmission&&o.enable(15),_.sheen&&o.enable(16),_.opaque&&o.enable(17),_.pointsUvs&&o.enable(18),_.decodeVideoTexture&&o.enable(19),_.decodeVideoTextureEmissive&&o.enable(20),_.alphaToCoverage&&o.enable(21),w.push(o.mask)}function y(w){let _=g[w.type],R;if(_){let G=Yn[_];R=ii.clone(G.uniforms)}else R=w.uniforms;return R}function D(w,_){let R;for(let G=0,N=h.length;G<N;G++){let z=h[G];if(z.cacheKey===_){R=z,++R.usedTimes;break}}return R===void 0&&(R=new P1(n,_,w,r),h.push(R)),R}function S(w){if(--w.usedTimes===0){let _=h.indexOf(w);h[_]=h[h.length-1],h.pop(),w.destroy()}}function C(w){l.remove(w)}function P(){l.dispose()}return{getParameters:m,getProgramCacheKey:p,getUniforms:y,acquireProgram:D,releaseProgram:S,releaseShaderCache:C,programs:h,dispose:P}}function D1(){let n=new WeakMap;function t(a){return n.has(a)}function e(a){let o=n.get(a);return o===void 0&&(o={},n.set(a,o)),o}function i(a){n.delete(a)}function s(a,o,l){n.get(a)[o]=l}function r(){n=new WeakMap}return{has:t,get:e,remove:i,update:s,dispose:r}}function k1(n,t){return n.groupOrder!==t.groupOrder?n.groupOrder-t.groupOrder:n.renderOrder!==t.renderOrder?n.renderOrder-t.renderOrder:n.material.id!==t.material.id?n.material.id-t.material.id:n.z!==t.z?n.z-t.z:n.id-t.id}function Vd(n,t){return n.groupOrder!==t.groupOrder?n.groupOrder-t.groupOrder:n.renderOrder!==t.renderOrder?n.renderOrder-t.renderOrder:n.z!==t.z?t.z-n.z:n.id-t.id}function Gd(){let n=[],t=0,e=[],i=[],s=[];function r(){t=0,e.length=0,i.length=0,s.length=0}function a(u,d,f,g,x,m){let p=n[t];return p===void 0?(p={id:u.id,object:u,geometry:d,material:f,groupOrder:g,renderOrder:u.renderOrder,z:x,group:m},n[t]=p):(p.id=u.id,p.object=u,p.geometry=d,p.material=f,p.groupOrder=g,p.renderOrder=u.renderOrder,p.z=x,p.group=m),t++,p}function o(u,d,f,g,x,m){let p=a(u,d,f,g,x,m);f.transmission>0?i.push(p):f.transparent===!0?s.push(p):e.push(p)}function l(u,d,f,g,x,m){let p=a(u,d,f,g,x,m);f.transmission>0?i.unshift(p):f.transparent===!0?s.unshift(p):e.unshift(p)}function c(u,d){e.length>1&&e.sort(u||k1),i.length>1&&i.sort(d||Vd),s.length>1&&s.sort(d||Vd)}function h(){for(let u=t,d=n.length;u<d;u++){let f=n[u];if(f.id===null)break;f.id=null,f.object=null,f.geometry=null,f.material=null,f.group=null}}return{opaque:e,transmissive:i,transparent:s,init:r,push:o,unshift:l,finish:h,sort:c}}function L1(){let n=new WeakMap;function t(i,s){let r=n.get(i),a;return r===void 0?(a=new Gd,n.set(i,[a])):s>=r.length?(a=new Gd,r.push(a)):a=r[s],a}function e(){n=new WeakMap}return{get:t,dispose:e}}function U1(){let n={};return{get:function(t){if(n[t.id]!==void 0)return n[t.id];let e;switch(t.type){case"DirectionalLight":e={direction:new I,color:new Pt};break;case"SpotLight":e={position:new I,direction:new I,color:new Pt,distance:0,coneCos:0,penumbraCos:0,decay:0};break;case"PointLight":e={position:new I,color:new Pt,distance:0,decay:0};break;case"HemisphereLight":e={direction:new I,skyColor:new Pt,groundColor:new Pt};break;case"RectAreaLight":e={color:new Pt,position:new I,halfWidth:new I,halfHeight:new I};break}return n[t.id]=e,e}}}function N1(){let n={};return{get:function(t){if(n[t.id]!==void 0)return n[t.id];let e;switch(t.type){case"DirectionalLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new j};break;case"SpotLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new j};break;case"PointLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new j,shadowCameraNear:1,shadowCameraFar:1e3};break}return n[t.id]=e,e}}}var B1=0;function O1(n,t){return(t.castShadow?2:0)-(n.castShadow?2:0)+(t.map?1:0)-(n.map?1:0)}function z1(n){let t=new U1,e=N1(),i={version:0,hash:{directionalLength:-1,pointLength:-1,spotLength:-1,rectAreaLength:-1,hemiLength:-1,numDirectionalShadows:-1,numPointShadows:-1,numSpotShadows:-1,numSpotMaps:-1,numLightProbes:-1},ambient:[0,0,0],probe:[],directional:[],directionalShadow:[],directionalShadowMap:[],directionalShadowMatrix:[],spot:[],spotLightMap:[],spotShadow:[],spotShadowMap:[],spotLightMatrix:[],rectArea:[],rectAreaLTC1:null,rectAreaLTC2:null,point:[],pointShadow:[],pointShadowMap:[],pointShadowMatrix:[],hemi:[],numSpotLightShadowsWithMaps:0,numLightProbes:0};for(let c=0;c<9;c++)i.probe.push(new I);let s=new I,r=new pe,a=new pe;function o(c){let h=0,u=0,d=0;for(let w=0;w<9;w++)i.probe[w].set(0,0,0);let f=0,g=0,x=0,m=0,p=0,T=0,M=0,y=0,D=0,S=0,C=0;c.sort(O1);for(let w=0,_=c.length;w<_;w++){let R=c[w],G=R.color,N=R.intensity,z=R.distance,J=R.shadow&&R.shadow.map?R.shadow.map.texture:null;if(R.isAmbientLight)h+=G.r*N,u+=G.g*N,d+=G.b*N;else if(R.isLightProbe){for(let V=0;V<9;V++)i.probe[V].addScaledVector(R.sh.coefficients[V],N);C++}else if(R.isDirectionalLight){let V=t.get(R);if(V.color.copy(R.color).multiplyScalar(R.intensity),R.castShadow){let it=R.shadow,W=e.get(R);W.shadowIntensity=it.intensity,W.shadowBias=it.bias,W.shadowNormalBias=it.normalBias,W.shadowRadius=it.radius,W.shadowMapSize=it.mapSize,i.directionalShadow[f]=W,i.directionalShadowMap[f]=J,i.directionalShadowMatrix[f]=R.shadow.matrix,T++}i.directional[f]=V,f++}else if(R.isSpotLight){let V=t.get(R);V.position.setFromMatrixPosition(R.matrixWorld),V.color.copy(G).multiplyScalar(N),V.distance=z,V.coneCos=Math.cos(R.angle),V.penumbraCos=Math.cos(R.angle*(1-R.penumbra)),V.decay=R.decay,i.spot[x]=V;let it=R.shadow;if(R.map&&(i.spotLightMap[D]=R.map,D++,it.updateMatrices(R),R.castShadow&&S++),i.spotLightMatrix[x]=it.matrix,R.castShadow){let W=e.get(R);W.shadowIntensity=it.intensity,W.shadowBias=it.bias,W.shadowNormalBias=it.normalBias,W.shadowRadius=it.radius,W.shadowMapSize=it.mapSize,i.spotShadow[x]=W,i.spotShadowMap[x]=J,y++}x++}else if(R.isRectAreaLight){let V=t.get(R);V.color.copy(G).multiplyScalar(N),V.halfWidth.set(R.width*.5,0,0),V.halfHeight.set(0,R.height*.5,0),i.rectArea[m]=V,m++}else if(R.isPointLight){let V=t.get(R);if(V.color.copy(R.color).multiplyScalar(R.intensity),V.distance=R.distance,V.decay=R.decay,R.castShadow){let it=R.shadow,W=e.get(R);W.shadowIntensity=it.intensity,W.shadowBias=it.bias,W.shadowNormalBias=it.normalBias,W.shadowRadius=it.radius,W.shadowMapSize=it.mapSize,W.shadowCameraNear=it.camera.near,W.shadowCameraFar=it.camera.far,i.pointShadow[g]=W,i.pointShadowMap[g]=J,i.pointShadowMatrix[g]=R.shadow.matrix,M++}i.point[g]=V,g++}else if(R.isHemisphereLight){let V=t.get(R);V.skyColor.copy(R.color).multiplyScalar(N),V.groundColor.copy(R.groundColor).multiplyScalar(N),i.hemi[p]=V,p++}}m>0&&(n.has("OES_texture_float_linear")===!0?(i.rectAreaLTC1=ht.LTC_FLOAT_1,i.rectAreaLTC2=ht.LTC_FLOAT_2):(i.rectAreaLTC1=ht.LTC_HALF_1,i.rectAreaLTC2=ht.LTC_HALF_2)),i.ambient[0]=h,i.ambient[1]=u,i.ambient[2]=d;let P=i.hash;(P.directionalLength!==f||P.pointLength!==g||P.spotLength!==x||P.rectAreaLength!==m||P.hemiLength!==p||P.numDirectionalShadows!==T||P.numPointShadows!==M||P.numSpotShadows!==y||P.numSpotMaps!==D||P.numLightProbes!==C)&&(i.directional.length=f,i.spot.length=x,i.rectArea.length=m,i.point.length=g,i.hemi.length=p,i.directionalShadow.length=T,i.directionalShadowMap.length=T,i.pointShadow.length=M,i.pointShadowMap.length=M,i.spotShadow.length=y,i.spotShadowMap.length=y,i.directionalShadowMatrix.length=T,i.pointShadowMatrix.length=M,i.spotLightMatrix.length=y+D-S,i.spotLightMap.length=D,i.numSpotLightShadowsWithMaps=S,i.numLightProbes=C,P.directionalLength=f,P.pointLength=g,P.spotLength=x,P.rectAreaLength=m,P.hemiLength=p,P.numDirectionalShadows=T,P.numPointShadows=M,P.numSpotShadows=y,P.numSpotMaps=D,P.numLightProbes=C,i.version=B1++)}function l(c,h){let u=0,d=0,f=0,g=0,x=0,m=h.matrixWorldInverse;for(let p=0,T=c.length;p<T;p++){let M=c[p];if(M.isDirectionalLight){let y=i.directional[u];y.direction.setFromMatrixPosition(M.matrixWorld),s.setFromMatrixPosition(M.target.matrixWorld),y.direction.sub(s),y.direction.transformDirection(m),u++}else if(M.isSpotLight){let y=i.spot[f];y.position.setFromMatrixPosition(M.matrixWorld),y.position.applyMatrix4(m),y.direction.setFromMatrixPosition(M.matrixWorld),s.setFromMatrixPosition(M.target.matrixWorld),y.direction.sub(s),y.direction.transformDirection(m),f++}else if(M.isRectAreaLight){let y=i.rectArea[g];y.position.setFromMatrixPosition(M.matrixWorld),y.position.applyMatrix4(m),a.identity(),r.copy(M.matrixWorld),r.premultiply(m),a.extractRotation(r),y.halfWidth.set(M.width*.5,0,0),y.halfHeight.set(0,M.height*.5,0),y.halfWidth.applyMatrix4(a),y.halfHeight.applyMatrix4(a),g++}else if(M.isPointLight){let y=i.point[d];y.position.setFromMatrixPosition(M.matrixWorld),y.position.applyMatrix4(m),d++}else if(M.isHemisphereLight){let y=i.hemi[x];y.direction.setFromMatrixPosition(M.matrixWorld),y.direction.transformDirection(m),x++}}}return{setup:o,setupView:l,state:i}}function Wd(n){let t=new z1(n),e=[],i=[];function s(h){c.camera=h,e.length=0,i.length=0}function r(h){e.push(h)}function a(h){i.push(h)}function o(){t.setup(e)}function l(h){t.setupView(e,h)}let c={lightsArray:e,shadowsArray:i,camera:null,lights:t,transmissionRenderTarget:{}};return{init:s,state:c,setupLights:o,setupLightsView:l,pushLight:r,pushShadow:a}}function H1(n){let t=new WeakMap;function e(s,r=0){let a=t.get(s),o;return a===void 0?(o=new Wd(n),t.set(s,[o])):r>=a.length?(o=new Wd(n),a.push(o)):o=a[r],o}function i(){t=new WeakMap}return{get:e,dispose:i}}var _h=class extends es{static get type(){return"MeshDepthMaterial"}constructor(t){super(),this.isMeshDepthMaterial=!0,this.depthPacking=Zp,this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.wireframe=!1,this.wireframeLinewidth=1,this.setValues(t)}copy(t){return super.copy(t),this.depthPacking=t.depthPacking,this.map=t.map,this.alphaMap=t.alphaMap,this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this}},bh=class extends es{static get type(){return"MeshDistanceMaterial"}constructor(t){super(),this.isMeshDistanceMaterial=!0,this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.setValues(t)}copy(t){return super.copy(t),this.map=t.map,this.alphaMap=t.alphaMap,this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this}},V1=`void main() {
	gl_Position = vec4( position, 1.0 );
}`,G1=`uniform sampler2D shadow_pass;
uniform vec2 resolution;
uniform float radius;
#include <packing>
void main() {
	const float samples = float( VSM_SAMPLES );
	float mean = 0.0;
	float squared_mean = 0.0;
	float uvStride = samples <= 1.0 ? 0.0 : 2.0 / ( samples - 1.0 );
	float uvStart = samples <= 1.0 ? 0.0 : - 1.0;
	for ( float i = 0.0; i < samples; i ++ ) {
		float uvOffset = uvStart + i * uvStride;
		#ifdef HORIZONTAL_PASS
			vec2 distribution = unpackRGBATo2Half( texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( uvOffset, 0.0 ) * radius ) / resolution ) );
			mean += distribution.x;
			squared_mean += distribution.y * distribution.y + distribution.x * distribution.x;
		#else
			float depth = unpackRGBAToDepth( texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( 0.0, uvOffset ) * radius ) / resolution ) );
			mean += depth;
			squared_mean += depth * depth;
		#endif
	}
	mean = mean / samples;
	squared_mean = squared_mean / samples;
	float std_dev = sqrt( squared_mean - mean * mean );
	gl_FragColor = pack2HalfToRGBA( vec2( mean, std_dev ) );
}`;function W1(n,t,e){let i=new pa,s=new j,r=new j,a=new xe,o=new _h({depthPacking:Jp}),l=new bh,c={},h=e.maxTextureSize,u={[Qi]:je,[je]:Qi,[vn]:vn},d=new Te({defines:{VSM_SAMPLES:8},uniforms:{shadow_pass:{value:null},resolution:{value:new j},radius:{value:4}},vertexShader:V1,fragmentShader:G1}),f=d.clone();f.defines.HORIZONTAL_PASS=1;let g=new tn;g.setAttribute("position",new an(new Float32Array([-1,-1,.5,3,-1,.5,-1,3,.5]),3));let x=new Ot(g,d),m=this;this.enabled=!1,this.autoUpdate=!0,this.needsUpdate=!1,this.type=af;let p=this.type;this.render=function(S,C,P){if(m.enabled===!1||m.autoUpdate===!1&&m.needsUpdate===!1||S.length===0)return;let w=n.getRenderTarget(),_=n.getActiveCubeFace(),R=n.getActiveMipmapLevel(),G=n.state;G.setBlending(Rn),G.buffers.color.setClear(1,1,1,1),G.buffers.depth.setTest(!0),G.setScissorTest(!1);let N=p!==bi&&this.type===bi,z=p===bi&&this.type!==bi;for(let J=0,V=S.length;J<V;J++){let it=S[J],W=it.shadow;if(W===void 0){console.warn("THREE.WebGLShadowMap:",it,"has no shadow.");continue}if(W.autoUpdate===!1&&W.needsUpdate===!1)continue;s.copy(W.mapSize);let ct=W.getFrameExtents();if(s.multiply(ct),r.copy(W.mapSize),(s.x>h||s.y>h)&&(s.x>h&&(r.x=Math.floor(h/ct.x),s.x=r.x*ct.x,W.mapSize.x=r.x),s.y>h&&(r.y=Math.floor(h/ct.y),s.y=r.y*ct.y,W.mapSize.y=r.y)),W.map===null||N===!0||z===!0){let Mt=this.type!==bi?{minFilter:yn,magFilter:yn}:{};W.map!==null&&W.map.dispose(),W.map=new Ue(s.x,s.y,Mt),W.map.texture.name=it.name+".shadowMap",W.camera.updateProjectionMatrix()}n.setRenderTarget(W.map),n.clear();let gt=W.getViewportCount();for(let Mt=0;Mt<gt;Mt++){let Yt=W.getViewport(Mt);a.set(r.x*Yt.x,r.y*Yt.y,r.x*Yt.z,r.y*Yt.w),G.viewport(a),W.updateMatrices(it,Mt),i=W.getFrustum(),y(C,P,W.camera,it,this.type)}W.isPointLightShadow!==!0&&this.type===bi&&T(W,P),W.needsUpdate=!1}p=this.type,m.needsUpdate=!1,n.setRenderTarget(w,_,R)};function T(S,C){let P=t.update(x);d.defines.VSM_SAMPLES!==S.blurSamples&&(d.defines.VSM_SAMPLES=S.blurSamples,f.defines.VSM_SAMPLES=S.blurSamples,d.needsUpdate=!0,f.needsUpdate=!0),S.mapPass===null&&(S.mapPass=new Ue(s.x,s.y)),d.uniforms.shadow_pass.value=S.map.texture,d.uniforms.resolution.value=S.mapSize,d.uniforms.radius.value=S.radius,n.setRenderTarget(S.mapPass),n.clear(),n.renderBufferDirect(C,null,P,d,x,null),f.uniforms.shadow_pass.value=S.mapPass.texture,f.uniforms.resolution.value=S.mapSize,f.uniforms.radius.value=S.radius,n.setRenderTarget(S.map),n.clear(),n.renderBufferDirect(C,null,P,f,x,null)}function M(S,C,P,w){let _=null,R=P.isPointLight===!0?S.customDistanceMaterial:S.customDepthMaterial;if(R!==void 0)_=R;else if(_=P.isPointLight===!0?l:o,n.localClippingEnabled&&C.clipShadows===!0&&Array.isArray(C.clippingPlanes)&&C.clippingPlanes.length!==0||C.displacementMap&&C.displacementScale!==0||C.alphaMap&&C.alphaTest>0||C.map&&C.alphaTest>0){let G=_.uuid,N=C.uuid,z=c[G];z===void 0&&(z={},c[G]=z);let J=z[N];J===void 0&&(J=_.clone(),z[N]=J,C.addEventListener("dispose",D)),_=J}if(_.visible=C.visible,_.wireframe=C.wireframe,w===bi?_.side=C.shadowSide!==null?C.shadowSide:C.side:_.side=C.shadowSide!==null?C.shadowSide:u[C.side],_.alphaMap=C.alphaMap,_.alphaTest=C.alphaTest,_.map=C.map,_.clipShadows=C.clipShadows,_.clippingPlanes=C.clippingPlanes,_.clipIntersection=C.clipIntersection,_.displacementMap=C.displacementMap,_.displacementScale=C.displacementScale,_.displacementBias=C.displacementBias,_.wireframeLinewidth=C.wireframeLinewidth,_.linewidth=C.linewidth,P.isPointLight===!0&&_.isMeshDistanceMaterial===!0){let G=n.properties.get(_);G.light=P}return _}function y(S,C,P,w,_){if(S.visible===!1)return;if(S.layers.test(C.layers)&&(S.isMesh||S.isLine||S.isPoints)&&(S.castShadow||S.receiveShadow&&_===bi)&&(!S.frustumCulled||i.intersectsObject(S))){S.modelViewMatrix.multiplyMatrices(P.matrixWorldInverse,S.matrixWorld);let N=t.update(S),z=S.material;if(Array.isArray(z)){let J=N.groups;for(let V=0,it=J.length;V<it;V++){let W=J[V],ct=z[W.materialIndex];if(ct&&ct.visible){let gt=M(S,ct,w,_);S.onBeforeShadow(n,S,C,P,N,gt,W),n.renderBufferDirect(P,null,N,gt,S,W),S.onAfterShadow(n,S,C,P,N,gt,W)}}}else if(z.visible){let J=M(S,z,w,_);S.onBeforeShadow(n,S,C,P,N,J,null),n.renderBufferDirect(P,null,N,J,S,null),S.onAfterShadow(n,S,C,P,N,J,null)}}let G=S.children;for(let N=0,z=G.length;N<z;N++)y(G[N],C,P,w,_)}function D(S){S.target.removeEventListener("dispose",D);for(let P in c){let w=c[P],_=S.target.uuid;_ in w&&(w[_].dispose(),delete w[_])}}}var q1={[Mc]:Sc,[wc]:Ac,[Ec]:Cc,[Er]:Tc,[Sc]:Mc,[Ac]:wc,[Cc]:Ec,[Tc]:Er};function X1(n,t){function e(){let F=!1,dt=new xe,q=null,K=new xe(0,0,0,0);return{setMask:function(mt){q!==mt&&!F&&(n.colorMask(mt,mt,mt,mt),q=mt)},setLocked:function(mt){F=mt},setClear:function(mt,ft,Zt,Ie,Ye){Ye===!0&&(mt*=Ie,ft*=Ie,Zt*=Ie),dt.set(mt,ft,Zt,Ie),K.equals(dt)===!1&&(n.clearColor(mt,ft,Zt,Ie),K.copy(dt))},reset:function(){F=!1,q=null,K.set(-1,0,0,0)}}}function i(){let F=!1,dt=!1,q=null,K=null,mt=null;return{setReversed:function(ft){if(dt!==ft){let Zt=t.get("EXT_clip_control");dt?Zt.clipControlEXT(Zt.LOWER_LEFT_EXT,Zt.ZERO_TO_ONE_EXT):Zt.clipControlEXT(Zt.LOWER_LEFT_EXT,Zt.NEGATIVE_ONE_TO_ONE_EXT);let Ie=mt;mt=null,this.setClear(Ie)}dt=ft},getReversed:function(){return dt},setTest:function(ft){ft?ot(n.DEPTH_TEST):Bt(n.DEPTH_TEST)},setMask:function(ft){q!==ft&&!F&&(n.depthMask(ft),q=ft)},setFunc:function(ft){if(dt&&(ft=q1[ft]),K!==ft){switch(ft){case Mc:n.depthFunc(n.NEVER);break;case Sc:n.depthFunc(n.ALWAYS);break;case wc:n.depthFunc(n.LESS);break;case Er:n.depthFunc(n.LEQUAL);break;case Ec:n.depthFunc(n.EQUAL);break;case Tc:n.depthFunc(n.GEQUAL);break;case Ac:n.depthFunc(n.GREATER);break;case Cc:n.depthFunc(n.NOTEQUAL);break;default:n.depthFunc(n.LEQUAL)}K=ft}},setLocked:function(ft){F=ft},setClear:function(ft){mt!==ft&&(dt&&(ft=1-ft),n.clearDepth(ft),mt=ft)},reset:function(){F=!1,q=null,K=null,mt=null,dt=!1}}}function s(){let F=!1,dt=null,q=null,K=null,mt=null,ft=null,Zt=null,Ie=null,Ye=null;return{setTest:function(me){F||(me?ot(n.STENCIL_TEST):Bt(n.STENCIL_TEST))},setMask:function(me){dt!==me&&!F&&(n.stencilMask(me),dt=me)},setFunc:function(me,Un,pi){(q!==me||K!==Un||mt!==pi)&&(n.stencilFunc(me,Un,pi),q=me,K=Un,mt=pi)},setOp:function(me,Un,pi){(ft!==me||Zt!==Un||Ie!==pi)&&(n.stencilOp(me,Un,pi),ft=me,Zt=Un,Ie=pi)},setLocked:function(me){F=me},setClear:function(me){Ye!==me&&(n.clearStencil(me),Ye=me)},reset:function(){F=!1,dt=null,q=null,K=null,mt=null,ft=null,Zt=null,Ie=null,Ye=null}}}let r=new e,a=new i,o=new s,l=new WeakMap,c=new WeakMap,h={},u={},d=new WeakMap,f=[],g=null,x=!1,m=null,p=null,T=null,M=null,y=null,D=null,S=null,C=new Pt(0,0,0),P=0,w=!1,_=null,R=null,G=null,N=null,z=null,J=n.getParameter(n.MAX_COMBINED_TEXTURE_IMAGE_UNITS),V=!1,it=0,W=n.getParameter(n.VERSION);W.indexOf("WebGL")!==-1?(it=parseFloat(/^WebGL (\d)/.exec(W)[1]),V=it>=1):W.indexOf("OpenGL ES")!==-1&&(it=parseFloat(/^OpenGL ES (\d)/.exec(W)[1]),V=it>=2);let ct=null,gt={},Mt=n.getParameter(n.SCISSOR_BOX),Yt=n.getParameter(n.VIEWPORT),he=new xe().fromArray(Mt),Y=new xe().fromArray(Yt);function rt(F,dt,q,K){let mt=new Uint8Array(4),ft=n.createTexture();n.bindTexture(F,ft),n.texParameteri(F,n.TEXTURE_MIN_FILTER,n.NEAREST),n.texParameteri(F,n.TEXTURE_MAG_FILTER,n.NEAREST);for(let Zt=0;Zt<q;Zt++)F===n.TEXTURE_3D||F===n.TEXTURE_2D_ARRAY?n.texImage3D(dt,0,n.RGBA,1,1,K,0,n.RGBA,n.UNSIGNED_BYTE,mt):n.texImage2D(dt+Zt,0,n.RGBA,1,1,0,n.RGBA,n.UNSIGNED_BYTE,mt);return ft}let St={};St[n.TEXTURE_2D]=rt(n.TEXTURE_2D,n.TEXTURE_2D,1),St[n.TEXTURE_CUBE_MAP]=rt(n.TEXTURE_CUBE_MAP,n.TEXTURE_CUBE_MAP_POSITIVE_X,6),St[n.TEXTURE_2D_ARRAY]=rt(n.TEXTURE_2D_ARRAY,n.TEXTURE_2D_ARRAY,1,1),St[n.TEXTURE_3D]=rt(n.TEXTURE_3D,n.TEXTURE_3D,1,1),r.setClear(0,0,0,1),a.setClear(1),o.setClear(0),ot(n.DEPTH_TEST),a.setFunc(Er),et(!1),yt(Yu),ot(n.CULL_FACE),A(Rn);function ot(F){h[F]!==!0&&(n.enable(F),h[F]=!0)}function Bt(F){h[F]!==!1&&(n.disable(F),h[F]=!1)}function qt(F,dt){return u[F]!==dt?(n.bindFramebuffer(F,dt),u[F]=dt,F===n.DRAW_FRAMEBUFFER&&(u[n.FRAMEBUFFER]=dt),F===n.FRAMEBUFFER&&(u[n.DRAW_FRAMEBUFFER]=dt),!0):!1}function Vt(F,dt){let q=f,K=!1;if(F){q=d.get(dt),q===void 0&&(q=[],d.set(dt,q));let mt=F.textures;if(q.length!==mt.length||q[0]!==n.COLOR_ATTACHMENT0){for(let ft=0,Zt=mt.length;ft<Zt;ft++)q[ft]=n.COLOR_ATTACHMENT0+ft;q.length=mt.length,K=!0}}else q[0]!==n.BACK&&(q[0]=n.BACK,K=!0);K&&n.drawBuffers(q)}function le(F){return g!==F?(n.useProgram(F),g=F,!0):!1}let Q={[Si]:n.FUNC_ADD,[Pp]:n.FUNC_SUBTRACT,[Ip]:n.FUNC_REVERSE_SUBTRACT};Q[Fp]=n.MIN,Q[Dp]=n.MAX;let st={[Zh]:n.ZERO,[il]:n.ONE,[kp]:n.SRC_COLOR,[ua]:n.SRC_ALPHA,[zp]:n.SRC_ALPHA_SATURATE,[Bp]:n.DST_COLOR,[Up]:n.DST_ALPHA,[Lp]:n.ONE_MINUS_SRC_COLOR,[bc]:n.ONE_MINUS_SRC_ALPHA,[Op]:n.ONE_MINUS_DST_COLOR,[Np]:n.ONE_MINUS_DST_ALPHA,[Hp]:n.CONSTANT_COLOR,[Vp]:n.ONE_MINUS_CONSTANT_COLOR,[Gp]:n.CONSTANT_ALPHA,[Wp]:n.ONE_MINUS_CONSTANT_ALPHA};function A(F,dt,q,K,mt,ft,Zt,Ie,Ye,me){if(F===Rn){x===!0&&(Bt(n.BLEND),x=!1);return}if(x===!1&&(ot(n.BLEND),x=!0),F!==Yh){if(F!==m||me!==w){if((p!==Si||y!==Si)&&(n.blendEquation(n.FUNC_ADD),p=Si,y=Si),me)switch(F){case br:n.blendFuncSeparate(n.ONE,n.ONE_MINUS_SRC_ALPHA,n.ONE,n.ONE_MINUS_SRC_ALPHA);break;case Qn:n.blendFunc(n.ONE,n.ONE);break;case Zu:n.blendFuncSeparate(n.ZERO,n.ONE_MINUS_SRC_COLOR,n.ZERO,n.ONE);break;case Ju:n.blendFuncSeparate(n.ZERO,n.SRC_COLOR,n.ZERO,n.SRC_ALPHA);break;default:console.error("THREE.WebGLState: Invalid blending: ",F);break}else switch(F){case br:n.blendFuncSeparate(n.SRC_ALPHA,n.ONE_MINUS_SRC_ALPHA,n.ONE,n.ONE_MINUS_SRC_ALPHA);break;case Qn:n.blendFunc(n.SRC_ALPHA,n.ONE);break;case Zu:n.blendFuncSeparate(n.ZERO,n.ONE_MINUS_SRC_COLOR,n.ZERO,n.ONE);break;case Ju:n.blendFunc(n.ZERO,n.SRC_COLOR);break;default:console.error("THREE.WebGLState: Invalid blending: ",F);break}T=null,M=null,D=null,S=null,C.set(0,0,0),P=0,m=F,w=me}return}mt=mt||dt,ft=ft||q,Zt=Zt||K,(dt!==p||mt!==y)&&(n.blendEquationSeparate(Q[dt],Q[mt]),p=dt,y=mt),(q!==T||K!==M||ft!==D||Zt!==S)&&(n.blendFuncSeparate(st[q],st[K],st[ft],st[Zt]),T=q,M=K,D=ft,S=Zt),(Ie.equals(C)===!1||Ye!==P)&&(n.blendColor(Ie.r,Ie.g,Ie.b,Ye),C.copy(Ie),P=Ye),m=F,w=!1}function Ft(F,dt){F.side===vn?Bt(n.CULL_FACE):ot(n.CULL_FACE);let q=F.side===je;dt&&(q=!q),et(q),F.blending===br&&F.transparent===!1?A(Rn):A(F.blending,F.blendEquation,F.blendSrc,F.blendDst,F.blendEquationAlpha,F.blendSrcAlpha,F.blendDstAlpha,F.blendColor,F.blendAlpha,F.premultipliedAlpha),a.setFunc(F.depthFunc),a.setTest(F.depthTest),a.setMask(F.depthWrite),r.setMask(F.colorWrite);let K=F.stencilWrite;o.setTest(K),K&&(o.setMask(F.stencilWriteMask),o.setFunc(F.stencilFunc,F.stencilRef,F.stencilFuncMask),o.setOp(F.stencilFail,F.stencilZFail,F.stencilZPass)),zt(F.polygonOffset,F.polygonOffsetFactor,F.polygonOffsetUnits),F.alphaToCoverage===!0?ot(n.SAMPLE_ALPHA_TO_COVERAGE):Bt(n.SAMPLE_ALPHA_TO_COVERAGE)}function et(F){_!==F&&(F?n.frontFace(n.CW):n.frontFace(n.CCW),_=F)}function yt(F){F!==Ap?(ot(n.CULL_FACE),F!==R&&(F===Yu?n.cullFace(n.BACK):F===Cp?n.cullFace(n.FRONT):n.cullFace(n.FRONT_AND_BACK))):Bt(n.CULL_FACE),R=F}function lt(F){F!==G&&(V&&n.lineWidth(F),G=F)}function zt(F,dt,q){F?(ot(n.POLYGON_OFFSET_FILL),(N!==dt||z!==q)&&(n.polygonOffset(dt,q),N=dt,z=q)):Bt(n.POLYGON_OFFSET_FILL)}function xt(F){F?ot(n.SCISSOR_TEST):Bt(n.SCISSOR_TEST)}function E(F){F===void 0&&(F=n.TEXTURE0+J-1),ct!==F&&(n.activeTexture(F),ct=F)}function v(F,dt,q){q===void 0&&(ct===null?q=n.TEXTURE0+J-1:q=ct);let K=gt[q];K===void 0&&(K={type:void 0,texture:void 0},gt[q]=K),(K.type!==F||K.texture!==dt)&&(ct!==q&&(n.activeTexture(q),ct=q),n.bindTexture(F,dt||St[F]),K.type=F,K.texture=dt)}function U(){let F=gt[ct];F!==void 0&&F.type!==void 0&&(n.bindTexture(F.type,null),F.type=void 0,F.texture=void 0)}function X(){try{n.compressedTexImage2D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function tt(){try{n.compressedTexImage3D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function $(){try{n.texSubImage2D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function Et(){try{n.texSubImage3D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function ut(){try{n.compressedTexSubImage2D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function vt(){try{n.compressedTexSubImage3D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function ne(){try{n.texStorage2D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function nt(){try{n.texStorage3D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function _t(){try{n.texImage2D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function Ht(){try{n.texImage3D.apply(n,arguments)}catch(F){console.error("THREE.WebGLState:",F)}}function Wt(F){he.equals(F)===!1&&(n.scissor(F.x,F.y,F.z,F.w),he.copy(F))}function bt(F){Y.equals(F)===!1&&(n.viewport(F.x,F.y,F.z,F.w),Y.copy(F))}function ae(F,dt){let q=c.get(dt);q===void 0&&(q=new WeakMap,c.set(dt,q));let K=q.get(F);K===void 0&&(K=n.getUniformBlockIndex(dt,F.name),q.set(F,K))}function jt(F,dt){let K=c.get(dt).get(F);l.get(dt)!==K&&(n.uniformBlockBinding(dt,K,F.__bindingPointIndex),l.set(dt,K))}function ve(){n.disable(n.BLEND),n.disable(n.CULL_FACE),n.disable(n.DEPTH_TEST),n.disable(n.POLYGON_OFFSET_FILL),n.disable(n.SCISSOR_TEST),n.disable(n.STENCIL_TEST),n.disable(n.SAMPLE_ALPHA_TO_COVERAGE),n.blendEquation(n.FUNC_ADD),n.blendFunc(n.ONE,n.ZERO),n.blendFuncSeparate(n.ONE,n.ZERO,n.ONE,n.ZERO),n.blendColor(0,0,0,0),n.colorMask(!0,!0,!0,!0),n.clearColor(0,0,0,0),n.depthMask(!0),n.depthFunc(n.LESS),a.setReversed(!1),n.clearDepth(1),n.stencilMask(4294967295),n.stencilFunc(n.ALWAYS,0,4294967295),n.stencilOp(n.KEEP,n.KEEP,n.KEEP),n.clearStencil(0),n.cullFace(n.BACK),n.frontFace(n.CCW),n.polygonOffset(0,0),n.activeTexture(n.TEXTURE0),n.bindFramebuffer(n.FRAMEBUFFER,null),n.bindFramebuffer(n.DRAW_FRAMEBUFFER,null),n.bindFramebuffer(n.READ_FRAMEBUFFER,null),n.useProgram(null),n.lineWidth(1),n.scissor(0,0,n.canvas.width,n.canvas.height),n.viewport(0,0,n.canvas.width,n.canvas.height),h={},ct=null,gt={},u={},d=new WeakMap,f=[],g=null,x=!1,m=null,p=null,T=null,M=null,y=null,D=null,S=null,C=new Pt(0,0,0),P=0,w=!1,_=null,R=null,G=null,N=null,z=null,he.set(0,0,n.canvas.width,n.canvas.height),Y.set(0,0,n.canvas.width,n.canvas.height),r.reset(),a.reset(),o.reset()}return{buffers:{color:r,depth:a,stencil:o},enable:ot,disable:Bt,bindFramebuffer:qt,drawBuffers:Vt,useProgram:le,setBlending:A,setMaterial:Ft,setFlipSided:et,setCullFace:yt,setLineWidth:lt,setPolygonOffset:zt,setScissorTest:xt,activeTexture:E,bindTexture:v,unbindTexture:U,compressedTexImage2D:X,compressedTexImage3D:tt,texImage2D:_t,texImage3D:Ht,updateUBOMapping:ae,uniformBlockBinding:jt,texStorage2D:ne,texStorage3D:nt,texSubImage2D:$,texSubImage3D:Et,compressedTexSubImage2D:ut,compressedTexSubImage3D:vt,scissor:Wt,viewport:bt,reset:ve}}function qd(n,t,e,i){let s=$1(i);switch(e){case df:return n*t;case pf:return n*t;case mf:return n*t*2;case su:return n*t/s.components*s.byteLength;case ru:return n*t/s.components*s.byteLength;case gf:return n*t*2/s.components*s.byteLength;case au:return n*t*2/s.components*s.byteLength;case ff:return n*t*3/s.components*s.byteLength;case Hn:return n*t*4/s.components*s.byteLength;case ou:return n*t*4/s.components*s.byteLength;case To:case Ao:return Math.floor((n+3)/4)*Math.floor((t+3)/4)*8;case Co:case Ro:return Math.floor((n+3)/4)*Math.floor((t+3)/4)*16;case kc:case Uc:return Math.max(n,16)*Math.max(t,8)/4;case Dc:case Lc:return Math.max(n,8)*Math.max(t,8)/2;case Nc:case Bc:return Math.floor((n+3)/4)*Math.floor((t+3)/4)*8;case Oc:return Math.floor((n+3)/4)*Math.floor((t+3)/4)*16;case zc:return Math.floor((n+3)/4)*Math.floor((t+3)/4)*16;case Hc:return Math.floor((n+4)/5)*Math.floor((t+3)/4)*16;case Vc:return Math.floor((n+4)/5)*Math.floor((t+4)/5)*16;case Gc:return Math.floor((n+5)/6)*Math.floor((t+4)/5)*16;case Wc:return Math.floor((n+5)/6)*Math.floor((t+5)/6)*16;case qc:return Math.floor((n+7)/8)*Math.floor((t+4)/5)*16;case Xc:return Math.floor((n+7)/8)*Math.floor((t+5)/6)*16;case $c:return Math.floor((n+7)/8)*Math.floor((t+7)/8)*16;case Yc:return Math.floor((n+9)/10)*Math.floor((t+4)/5)*16;case Zc:return Math.floor((n+9)/10)*Math.floor((t+5)/6)*16;case Jc:return Math.floor((n+9)/10)*Math.floor((t+7)/8)*16;case Kc:return Math.floor((n+9)/10)*Math.floor((t+9)/10)*16;case Qc:return Math.floor((n+11)/12)*Math.floor((t+9)/10)*16;case jc:return Math.floor((n+11)/12)*Math.floor((t+11)/12)*16;case Po:case th:case eh:return Math.ceil(n/4)*Math.ceil(t/4)*16;case xf:case nh:return Math.ceil(n/4)*Math.ceil(t/4)*8;case ih:case sh:return Math.ceil(n/4)*Math.ceil(t/4)*16}throw new Error(`Unable to determine texture byte length for ${e} format.`)}function $1(n){switch(n){case Ai:case cf:return{byteLength:1,components:1};case da:case hf:case en:return{byteLength:2,components:1};case nu:case iu:return{byteLength:2,components:4};case Us:case eu:case Kn:return{byteLength:4,components:1};case uf:return{byteLength:4,components:3}}throw new Error(`Unknown texture type ${n}.`)}function Y1(n,t,e,i,s,r,a){let o=t.has("WEBGL_multisampled_render_to_texture")?t.get("WEBGL_multisampled_render_to_texture"):null,l=typeof navigator>"u"?!1:/OculusBrowser/g.test(navigator.userAgent),c=new j,h=new WeakMap,u,d=new WeakMap,f=!1;try{f=typeof OffscreenCanvas<"u"&&new OffscreenCanvas(1,1).getContext("2d")!==null}catch{}function g(E,v){return f?new OffscreenCanvas(E,v):fa("canvas")}function x(E,v,U){let X=1,tt=xt(E);if((tt.width>U||tt.height>U)&&(X=U/Math.max(tt.width,tt.height)),X<1)if(typeof HTMLImageElement<"u"&&E instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&E instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&E instanceof ImageBitmap||typeof VideoFrame<"u"&&E instanceof VideoFrame){let $=Math.floor(X*tt.width),Et=Math.floor(X*tt.height);u===void 0&&(u=g($,Et));let ut=v?g($,Et):u;return ut.width=$,ut.height=Et,ut.getContext("2d").drawImage(E,0,0,$,Et),console.warn("THREE.WebGLRenderer: Texture has been resized from ("+tt.width+"x"+tt.height+") to ("+$+"x"+Et+")."),ut}else return"data"in E&&console.warn("THREE.WebGLRenderer: Image in DataTexture is too big ("+tt.width+"x"+tt.height+")."),E;return E}function m(E){return E.generateMipmaps}function p(E){n.generateMipmap(E)}function T(E){return E.isWebGLCubeRenderTarget?n.TEXTURE_CUBE_MAP:E.isWebGL3DRenderTarget?n.TEXTURE_3D:E.isWebGLArrayRenderTarget||E.isCompressedArrayTexture?n.TEXTURE_2D_ARRAY:n.TEXTURE_2D}function M(E,v,U,X,tt=!1){if(E!==null){if(n[E]!==void 0)return n[E];console.warn("THREE.WebGLRenderer: Attempt to use non-existing WebGL internal format '"+E+"'")}let $=v;if(v===n.RED&&(U===n.FLOAT&&($=n.R32F),U===n.HALF_FLOAT&&($=n.R16F),U===n.UNSIGNED_BYTE&&($=n.R8)),v===n.RED_INTEGER&&(U===n.UNSIGNED_BYTE&&($=n.R8UI),U===n.UNSIGNED_SHORT&&($=n.R16UI),U===n.UNSIGNED_INT&&($=n.R32UI),U===n.BYTE&&($=n.R8I),U===n.SHORT&&($=n.R16I),U===n.INT&&($=n.R32I)),v===n.RG&&(U===n.FLOAT&&($=n.RG32F),U===n.HALF_FLOAT&&($=n.RG16F),U===n.UNSIGNED_BYTE&&($=n.RG8)),v===n.RG_INTEGER&&(U===n.UNSIGNED_BYTE&&($=n.RG8UI),U===n.UNSIGNED_SHORT&&($=n.RG16UI),U===n.UNSIGNED_INT&&($=n.RG32UI),U===n.BYTE&&($=n.RG8I),U===n.SHORT&&($=n.RG16I),U===n.INT&&($=n.RG32I)),v===n.RGB_INTEGER&&(U===n.UNSIGNED_BYTE&&($=n.RGB8UI),U===n.UNSIGNED_SHORT&&($=n.RGB16UI),U===n.UNSIGNED_INT&&($=n.RGB32UI),U===n.BYTE&&($=n.RGB8I),U===n.SHORT&&($=n.RGB16I),U===n.INT&&($=n.RGB32I)),v===n.RGBA_INTEGER&&(U===n.UNSIGNED_BYTE&&($=n.RGBA8UI),U===n.UNSIGNED_SHORT&&($=n.RGBA16UI),U===n.UNSIGNED_INT&&($=n.RGBA32UI),U===n.BYTE&&($=n.RGBA8I),U===n.SHORT&&($=n.RGBA16I),U===n.INT&&($=n.RGBA32I)),v===n.RGB&&U===n.UNSIGNED_INT_5_9_9_9_REV&&($=n.RGB9_E5),v===n.RGBA){let Et=tt?rl:se.getTransfer(X);U===n.FLOAT&&($=n.RGBA32F),U===n.HALF_FLOAT&&($=n.RGBA16F),U===n.UNSIGNED_BYTE&&($=Et===fe?n.SRGB8_ALPHA8:n.RGBA8),U===n.UNSIGNED_SHORT_4_4_4_4&&($=n.RGBA4),U===n.UNSIGNED_SHORT_5_5_5_1&&($=n.RGB5_A1)}return($===n.R16F||$===n.R32F||$===n.RG16F||$===n.RG32F||$===n.RGBA16F||$===n.RGBA32F)&&t.get("EXT_color_buffer_float"),$}function y(E,v){let U;return E?v===null||v===Us||v===Cr?U=n.DEPTH24_STENCIL8:v===Kn?U=n.DEPTH32F_STENCIL8:v===da&&(U=n.DEPTH24_STENCIL8,console.warn("DepthTexture: 16 bit depth attachment is not supported with stencil. Using 24-bit attachment.")):v===null||v===Us||v===Cr?U=n.DEPTH_COMPONENT24:v===Kn?U=n.DEPTH_COMPONENT32F:v===da&&(U=n.DEPTH_COMPONENT16),U}function D(E,v){return m(E)===!0||E.isFramebufferTexture&&E.minFilter!==yn&&E.minFilter!==Jn?Math.log2(Math.max(v.width,v.height))+1:E.mipmaps!==void 0&&E.mipmaps.length>0?E.mipmaps.length:E.isCompressedTexture&&Array.isArray(E.image)?v.mipmaps.length:1}function S(E){let v=E.target;v.removeEventListener("dispose",S),P(v),v.isVideoTexture&&h.delete(v)}function C(E){let v=E.target;v.removeEventListener("dispose",C),_(v)}function P(E){let v=i.get(E);if(v.__webglInit===void 0)return;let U=E.source,X=d.get(U);if(X){let tt=X[v.__cacheKey];tt.usedTimes--,tt.usedTimes===0&&w(E),Object.keys(X).length===0&&d.delete(U)}i.remove(E)}function w(E){let v=i.get(E);n.deleteTexture(v.__webglTexture);let U=E.source,X=d.get(U);delete X[v.__cacheKey],a.memory.textures--}function _(E){let v=i.get(E);if(E.depthTexture&&(E.depthTexture.dispose(),i.remove(E.depthTexture)),E.isWebGLCubeRenderTarget)for(let X=0;X<6;X++){if(Array.isArray(v.__webglFramebuffer[X]))for(let tt=0;tt<v.__webglFramebuffer[X].length;tt++)n.deleteFramebuffer(v.__webglFramebuffer[X][tt]);else n.deleteFramebuffer(v.__webglFramebuffer[X]);v.__webglDepthbuffer&&n.deleteRenderbuffer(v.__webglDepthbuffer[X])}else{if(Array.isArray(v.__webglFramebuffer))for(let X=0;X<v.__webglFramebuffer.length;X++)n.deleteFramebuffer(v.__webglFramebuffer[X]);else n.deleteFramebuffer(v.__webglFramebuffer);if(v.__webglDepthbuffer&&n.deleteRenderbuffer(v.__webglDepthbuffer),v.__webglMultisampledFramebuffer&&n.deleteFramebuffer(v.__webglMultisampledFramebuffer),v.__webglColorRenderbuffer)for(let X=0;X<v.__webglColorRenderbuffer.length;X++)v.__webglColorRenderbuffer[X]&&n.deleteRenderbuffer(v.__webglColorRenderbuffer[X]);v.__webglDepthRenderbuffer&&n.deleteRenderbuffer(v.__webglDepthRenderbuffer)}let U=E.textures;for(let X=0,tt=U.length;X<tt;X++){let $=i.get(U[X]);$.__webglTexture&&(n.deleteTexture($.__webglTexture),a.memory.textures--),i.remove(U[X])}i.remove(E)}let R=0;function G(){R=0}function N(){let E=R;return E>=s.maxTextures&&console.warn("THREE.WebGLTextures: Trying to use "+E+" texture units while this GPU supports only "+s.maxTextures),R+=1,E}function z(E){let v=[];return v.push(E.wrapS),v.push(E.wrapT),v.push(E.wrapR||0),v.push(E.magFilter),v.push(E.minFilter),v.push(E.anisotropy),v.push(E.internalFormat),v.push(E.format),v.push(E.type),v.push(E.generateMipmaps),v.push(E.premultiplyAlpha),v.push(E.flipY),v.push(E.unpackAlignment),v.push(E.colorSpace),v.join()}function J(E,v){let U=i.get(E);if(E.isVideoTexture&&lt(E),E.isRenderTargetTexture===!1&&E.version>0&&U.__version!==E.version){let X=E.image;if(X===null)console.warn("THREE.WebGLRenderer: Texture marked for update but no image data found.");else if(X.complete===!1)console.warn("THREE.WebGLRenderer: Texture marked for update but image is incomplete");else{Y(U,E,v);return}}e.bindTexture(n.TEXTURE_2D,U.__webglTexture,n.TEXTURE0+v)}function V(E,v){let U=i.get(E);if(E.version>0&&U.__version!==E.version){Y(U,E,v);return}e.bindTexture(n.TEXTURE_2D_ARRAY,U.__webglTexture,n.TEXTURE0+v)}function it(E,v){let U=i.get(E);if(E.version>0&&U.__version!==E.version){Y(U,E,v);return}e.bindTexture(n.TEXTURE_3D,U.__webglTexture,n.TEXTURE0+v)}function W(E,v){let U=i.get(E);if(E.version>0&&U.__version!==E.version){rt(U,E,v);return}e.bindTexture(n.TEXTURE_CUBE_MAP,U.__webglTexture,n.TEXTURE0+v)}let ct={[Ic]:n.REPEAT,[ks]:n.CLAMP_TO_EDGE,[Fc]:n.MIRRORED_REPEAT},gt={[yn]:n.NEAREST,[Yp]:n.NEAREST_MIPMAP_NEAREST,[Ja]:n.NEAREST_MIPMAP_LINEAR,[Jn]:n.LINEAR,[Ol]:n.LINEAR_MIPMAP_NEAREST,[Ls]:n.LINEAR_MIPMAP_LINEAR},Mt={[Qp]:n.NEVER,[sm]:n.ALWAYS,[jp]:n.LESS,[yf]:n.LEQUAL,[tm]:n.EQUAL,[im]:n.GEQUAL,[em]:n.GREATER,[nm]:n.NOTEQUAL};function Yt(E,v){if(v.type===Kn&&t.has("OES_texture_float_linear")===!1&&(v.magFilter===Jn||v.magFilter===Ol||v.magFilter===Ja||v.magFilter===Ls||v.minFilter===Jn||v.minFilter===Ol||v.minFilter===Ja||v.minFilter===Ls)&&console.warn("THREE.WebGLRenderer: Unable to use linear filtering with floating point textures. OES_texture_float_linear not supported on this device."),n.texParameteri(E,n.TEXTURE_WRAP_S,ct[v.wrapS]),n.texParameteri(E,n.TEXTURE_WRAP_T,ct[v.wrapT]),(E===n.TEXTURE_3D||E===n.TEXTURE_2D_ARRAY)&&n.texParameteri(E,n.TEXTURE_WRAP_R,ct[v.wrapR]),n.texParameteri(E,n.TEXTURE_MAG_FILTER,gt[v.magFilter]),n.texParameteri(E,n.TEXTURE_MIN_FILTER,gt[v.minFilter]),v.compareFunction&&(n.texParameteri(E,n.TEXTURE_COMPARE_MODE,n.COMPARE_REF_TO_TEXTURE),n.texParameteri(E,n.TEXTURE_COMPARE_FUNC,Mt[v.compareFunction])),t.has("EXT_texture_filter_anisotropic")===!0){if(v.magFilter===yn||v.minFilter!==Ja&&v.minFilter!==Ls||v.type===Kn&&t.has("OES_texture_float_linear")===!1)return;if(v.anisotropy>1||i.get(v).__currentAnisotropy){let U=t.get("EXT_texture_filter_anisotropic");n.texParameterf(E,U.TEXTURE_MAX_ANISOTROPY_EXT,Math.min(v.anisotropy,s.getMaxAnisotropy())),i.get(v).__currentAnisotropy=v.anisotropy}}}function he(E,v){let U=!1;E.__webglInit===void 0&&(E.__webglInit=!0,v.addEventListener("dispose",S));let X=v.source,tt=d.get(X);tt===void 0&&(tt={},d.set(X,tt));let $=z(v);if($!==E.__cacheKey){tt[$]===void 0&&(tt[$]={texture:n.createTexture(),usedTimes:0},a.memory.textures++,U=!0),tt[$].usedTimes++;let Et=tt[E.__cacheKey];Et!==void 0&&(tt[E.__cacheKey].usedTimes--,Et.usedTimes===0&&w(v)),E.__cacheKey=$,E.__webglTexture=tt[$].texture}return U}function Y(E,v,U){let X=n.TEXTURE_2D;(v.isDataArrayTexture||v.isCompressedArrayTexture)&&(X=n.TEXTURE_2D_ARRAY),v.isData3DTexture&&(X=n.TEXTURE_3D);let tt=he(E,v),$=v.source;e.bindTexture(X,E.__webglTexture,n.TEXTURE0+U);let Et=i.get($);if($.version!==Et.__version||tt===!0){e.activeTexture(n.TEXTURE0+U);let ut=se.getPrimaries(se.workingColorSpace),vt=v.colorSpace===Yi?null:se.getPrimaries(v.colorSpace),ne=v.colorSpace===Yi||ut===vt?n.NONE:n.BROWSER_DEFAULT_WEBGL;n.pixelStorei(n.UNPACK_FLIP_Y_WEBGL,v.flipY),n.pixelStorei(n.UNPACK_PREMULTIPLY_ALPHA_WEBGL,v.premultiplyAlpha),n.pixelStorei(n.UNPACK_ALIGNMENT,v.unpackAlignment),n.pixelStorei(n.UNPACK_COLORSPACE_CONVERSION_WEBGL,ne);let nt=x(v.image,!1,s.maxTextureSize);nt=zt(v,nt);let _t=r.convert(v.format,v.colorSpace),Ht=r.convert(v.type),Wt=M(v.internalFormat,_t,Ht,v.colorSpace,v.isVideoTexture);Yt(X,v);let bt,ae=v.mipmaps,jt=v.isVideoTexture!==!0,ve=Et.__version===void 0||tt===!0,F=$.dataReady,dt=D(v,nt);if(v.isDepthTexture)Wt=y(v.format===Rr,v.type),ve&&(jt?e.texStorage2D(n.TEXTURE_2D,1,Wt,nt.width,nt.height):e.texImage2D(n.TEXTURE_2D,0,Wt,nt.width,nt.height,0,_t,Ht,null));else if(v.isDataTexture)if(ae.length>0){jt&&ve&&e.texStorage2D(n.TEXTURE_2D,dt,Wt,ae[0].width,ae[0].height);for(let q=0,K=ae.length;q<K;q++)bt=ae[q],jt?F&&e.texSubImage2D(n.TEXTURE_2D,q,0,0,bt.width,bt.height,_t,Ht,bt.data):e.texImage2D(n.TEXTURE_2D,q,Wt,bt.width,bt.height,0,_t,Ht,bt.data);v.generateMipmaps=!1}else jt?(ve&&e.texStorage2D(n.TEXTURE_2D,dt,Wt,nt.width,nt.height),F&&e.texSubImage2D(n.TEXTURE_2D,0,0,0,nt.width,nt.height,_t,Ht,nt.data)):e.texImage2D(n.TEXTURE_2D,0,Wt,nt.width,nt.height,0,_t,Ht,nt.data);else if(v.isCompressedTexture)if(v.isCompressedArrayTexture){jt&&ve&&e.texStorage3D(n.TEXTURE_2D_ARRAY,dt,Wt,ae[0].width,ae[0].height,nt.depth);for(let q=0,K=ae.length;q<K;q++)if(bt=ae[q],v.format!==Hn)if(_t!==null)if(jt){if(F)if(v.layerUpdates.size>0){let mt=qd(bt.width,bt.height,v.format,v.type);for(let ft of v.layerUpdates){let Zt=bt.data.subarray(ft*mt/bt.data.BYTES_PER_ELEMENT,(ft+1)*mt/bt.data.BYTES_PER_ELEMENT);e.compressedTexSubImage3D(n.TEXTURE_2D_ARRAY,q,0,0,ft,bt.width,bt.height,1,_t,Zt)}v.clearLayerUpdates()}else e.compressedTexSubImage3D(n.TEXTURE_2D_ARRAY,q,0,0,0,bt.width,bt.height,nt.depth,_t,bt.data)}else e.compressedTexImage3D(n.TEXTURE_2D_ARRAY,q,Wt,bt.width,bt.height,nt.depth,0,bt.data,0,0);else console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()");else jt?F&&e.texSubImage3D(n.TEXTURE_2D_ARRAY,q,0,0,0,bt.width,bt.height,nt.depth,_t,Ht,bt.data):e.texImage3D(n.TEXTURE_2D_ARRAY,q,Wt,bt.width,bt.height,nt.depth,0,_t,Ht,bt.data)}else{jt&&ve&&e.texStorage2D(n.TEXTURE_2D,dt,Wt,ae[0].width,ae[0].height);for(let q=0,K=ae.length;q<K;q++)bt=ae[q],v.format!==Hn?_t!==null?jt?F&&e.compressedTexSubImage2D(n.TEXTURE_2D,q,0,0,bt.width,bt.height,_t,bt.data):e.compressedTexImage2D(n.TEXTURE_2D,q,Wt,bt.width,bt.height,0,bt.data):console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()"):jt?F&&e.texSubImage2D(n.TEXTURE_2D,q,0,0,bt.width,bt.height,_t,Ht,bt.data):e.texImage2D(n.TEXTURE_2D,q,Wt,bt.width,bt.height,0,_t,Ht,bt.data)}else if(v.isDataArrayTexture)if(jt){if(ve&&e.texStorage3D(n.TEXTURE_2D_ARRAY,dt,Wt,nt.width,nt.height,nt.depth),F)if(v.layerUpdates.size>0){let q=qd(nt.width,nt.height,v.format,v.type);for(let K of v.layerUpdates){let mt=nt.data.subarray(K*q/nt.data.BYTES_PER_ELEMENT,(K+1)*q/nt.data.BYTES_PER_ELEMENT);e.texSubImage3D(n.TEXTURE_2D_ARRAY,0,0,0,K,nt.width,nt.height,1,_t,Ht,mt)}v.clearLayerUpdates()}else e.texSubImage3D(n.TEXTURE_2D_ARRAY,0,0,0,0,nt.width,nt.height,nt.depth,_t,Ht,nt.data)}else e.texImage3D(n.TEXTURE_2D_ARRAY,0,Wt,nt.width,nt.height,nt.depth,0,_t,Ht,nt.data);else if(v.isData3DTexture)jt?(ve&&e.texStorage3D(n.TEXTURE_3D,dt,Wt,nt.width,nt.height,nt.depth),F&&e.texSubImage3D(n.TEXTURE_3D,0,0,0,0,nt.width,nt.height,nt.depth,_t,Ht,nt.data)):e.texImage3D(n.TEXTURE_3D,0,Wt,nt.width,nt.height,nt.depth,0,_t,Ht,nt.data);else if(v.isFramebufferTexture){if(ve)if(jt)e.texStorage2D(n.TEXTURE_2D,dt,Wt,nt.width,nt.height);else{let q=nt.width,K=nt.height;for(let mt=0;mt<dt;mt++)e.texImage2D(n.TEXTURE_2D,mt,Wt,q,K,0,_t,Ht,null),q>>=1,K>>=1}}else if(ae.length>0){if(jt&&ve){let q=xt(ae[0]);e.texStorage2D(n.TEXTURE_2D,dt,Wt,q.width,q.height)}for(let q=0,K=ae.length;q<K;q++)bt=ae[q],jt?F&&e.texSubImage2D(n.TEXTURE_2D,q,0,0,_t,Ht,bt):e.texImage2D(n.TEXTURE_2D,q,Wt,_t,Ht,bt);v.generateMipmaps=!1}else if(jt){if(ve){let q=xt(nt);e.texStorage2D(n.TEXTURE_2D,dt,Wt,q.width,q.height)}F&&e.texSubImage2D(n.TEXTURE_2D,0,0,0,_t,Ht,nt)}else e.texImage2D(n.TEXTURE_2D,0,Wt,_t,Ht,nt);m(v)&&p(X),Et.__version=$.version,v.onUpdate&&v.onUpdate(v)}E.__version=v.version}function rt(E,v,U){if(v.image.length!==6)return;let X=he(E,v),tt=v.source;e.bindTexture(n.TEXTURE_CUBE_MAP,E.__webglTexture,n.TEXTURE0+U);let $=i.get(tt);if(tt.version!==$.__version||X===!0){e.activeTexture(n.TEXTURE0+U);let Et=se.getPrimaries(se.workingColorSpace),ut=v.colorSpace===Yi?null:se.getPrimaries(v.colorSpace),vt=v.colorSpace===Yi||Et===ut?n.NONE:n.BROWSER_DEFAULT_WEBGL;n.pixelStorei(n.UNPACK_FLIP_Y_WEBGL,v.flipY),n.pixelStorei(n.UNPACK_PREMULTIPLY_ALPHA_WEBGL,v.premultiplyAlpha),n.pixelStorei(n.UNPACK_ALIGNMENT,v.unpackAlignment),n.pixelStorei(n.UNPACK_COLORSPACE_CONVERSION_WEBGL,vt);let ne=v.isCompressedTexture||v.image[0].isCompressedTexture,nt=v.image[0]&&v.image[0].isDataTexture,_t=[];for(let K=0;K<6;K++)!ne&&!nt?_t[K]=x(v.image[K],!0,s.maxCubemapSize):_t[K]=nt?v.image[K].image:v.image[K],_t[K]=zt(v,_t[K]);let Ht=_t[0],Wt=r.convert(v.format,v.colorSpace),bt=r.convert(v.type),ae=M(v.internalFormat,Wt,bt,v.colorSpace),jt=v.isVideoTexture!==!0,ve=$.__version===void 0||X===!0,F=tt.dataReady,dt=D(v,Ht);Yt(n.TEXTURE_CUBE_MAP,v);let q;if(ne){jt&&ve&&e.texStorage2D(n.TEXTURE_CUBE_MAP,dt,ae,Ht.width,Ht.height);for(let K=0;K<6;K++){q=_t[K].mipmaps;for(let mt=0;mt<q.length;mt++){let ft=q[mt];v.format!==Hn?Wt!==null?jt?F&&e.compressedTexSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt,0,0,ft.width,ft.height,Wt,ft.data):e.compressedTexImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt,ae,ft.width,ft.height,0,ft.data):console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .setTextureCube()"):jt?F&&e.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt,0,0,ft.width,ft.height,Wt,bt,ft.data):e.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt,ae,ft.width,ft.height,0,Wt,bt,ft.data)}}}else{if(q=v.mipmaps,jt&&ve){q.length>0&&dt++;let K=xt(_t[0]);e.texStorage2D(n.TEXTURE_CUBE_MAP,dt,ae,K.width,K.height)}for(let K=0;K<6;K++)if(nt){jt?F&&e.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,0,0,0,_t[K].width,_t[K].height,Wt,bt,_t[K].data):e.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,0,ae,_t[K].width,_t[K].height,0,Wt,bt,_t[K].data);for(let mt=0;mt<q.length;mt++){let Zt=q[mt].image[K].image;jt?F&&e.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt+1,0,0,Zt.width,Zt.height,Wt,bt,Zt.data):e.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt+1,ae,Zt.width,Zt.height,0,Wt,bt,Zt.data)}}else{jt?F&&e.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,0,0,0,Wt,bt,_t[K]):e.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,0,ae,Wt,bt,_t[K]);for(let mt=0;mt<q.length;mt++){let ft=q[mt];jt?F&&e.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt+1,0,0,Wt,bt,ft.image[K]):e.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+K,mt+1,ae,Wt,bt,ft.image[K])}}}m(v)&&p(n.TEXTURE_CUBE_MAP),$.__version=tt.version,v.onUpdate&&v.onUpdate(v)}E.__version=v.version}function St(E,v,U,X,tt,$){let Et=r.convert(U.format,U.colorSpace),ut=r.convert(U.type),vt=M(U.internalFormat,Et,ut,U.colorSpace),ne=i.get(v),nt=i.get(U);if(nt.__renderTarget=v,!ne.__hasExternalTextures){let _t=Math.max(1,v.width>>$),Ht=Math.max(1,v.height>>$);tt===n.TEXTURE_3D||tt===n.TEXTURE_2D_ARRAY?e.texImage3D(tt,$,vt,_t,Ht,v.depth,0,Et,ut,null):e.texImage2D(tt,$,vt,_t,Ht,0,Et,ut,null)}e.bindFramebuffer(n.FRAMEBUFFER,E),yt(v)?o.framebufferTexture2DMultisampleEXT(n.FRAMEBUFFER,X,tt,nt.__webglTexture,0,et(v)):(tt===n.TEXTURE_2D||tt>=n.TEXTURE_CUBE_MAP_POSITIVE_X&&tt<=n.TEXTURE_CUBE_MAP_NEGATIVE_Z)&&n.framebufferTexture2D(n.FRAMEBUFFER,X,tt,nt.__webglTexture,$),e.bindFramebuffer(n.FRAMEBUFFER,null)}function ot(E,v,U){if(n.bindRenderbuffer(n.RENDERBUFFER,E),v.depthBuffer){let X=v.depthTexture,tt=X&&X.isDepthTexture?X.type:null,$=y(v.stencilBuffer,tt),Et=v.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,ut=et(v);yt(v)?o.renderbufferStorageMultisampleEXT(n.RENDERBUFFER,ut,$,v.width,v.height):U?n.renderbufferStorageMultisample(n.RENDERBUFFER,ut,$,v.width,v.height):n.renderbufferStorage(n.RENDERBUFFER,$,v.width,v.height),n.framebufferRenderbuffer(n.FRAMEBUFFER,Et,n.RENDERBUFFER,E)}else{let X=v.textures;for(let tt=0;tt<X.length;tt++){let $=X[tt],Et=r.convert($.format,$.colorSpace),ut=r.convert($.type),vt=M($.internalFormat,Et,ut,$.colorSpace),ne=et(v);U&&yt(v)===!1?n.renderbufferStorageMultisample(n.RENDERBUFFER,ne,vt,v.width,v.height):yt(v)?o.renderbufferStorageMultisampleEXT(n.RENDERBUFFER,ne,vt,v.width,v.height):n.renderbufferStorage(n.RENDERBUFFER,vt,v.width,v.height)}}n.bindRenderbuffer(n.RENDERBUFFER,null)}function Bt(E,v){if(v&&v.isWebGLCubeRenderTarget)throw new Error("Depth Texture with cube render targets is not supported");if(e.bindFramebuffer(n.FRAMEBUFFER,E),!(v.depthTexture&&v.depthTexture.isDepthTexture))throw new Error("renderTarget.depthTexture must be an instance of THREE.DepthTexture");let X=i.get(v.depthTexture);X.__renderTarget=v,(!X.__webglTexture||v.depthTexture.image.width!==v.width||v.depthTexture.image.height!==v.height)&&(v.depthTexture.image.width=v.width,v.depthTexture.image.height=v.height,v.depthTexture.needsUpdate=!0),J(v.depthTexture,0);let tt=X.__webglTexture,$=et(v);if(v.depthTexture.format===Mr)yt(v)?o.framebufferTexture2DMultisampleEXT(n.FRAMEBUFFER,n.DEPTH_ATTACHMENT,n.TEXTURE_2D,tt,0,$):n.framebufferTexture2D(n.FRAMEBUFFER,n.DEPTH_ATTACHMENT,n.TEXTURE_2D,tt,0);else if(v.depthTexture.format===Rr)yt(v)?o.framebufferTexture2DMultisampleEXT(n.FRAMEBUFFER,n.DEPTH_STENCIL_ATTACHMENT,n.TEXTURE_2D,tt,0,$):n.framebufferTexture2D(n.FRAMEBUFFER,n.DEPTH_STENCIL_ATTACHMENT,n.TEXTURE_2D,tt,0);else throw new Error("Unknown depthTexture format")}function qt(E){let v=i.get(E),U=E.isWebGLCubeRenderTarget===!0;if(v.__boundDepthTexture!==E.depthTexture){let X=E.depthTexture;if(v.__depthDisposeCallback&&v.__depthDisposeCallback(),X){let tt=()=>{delete v.__boundDepthTexture,delete v.__depthDisposeCallback,X.removeEventListener("dispose",tt)};X.addEventListener("dispose",tt),v.__depthDisposeCallback=tt}v.__boundDepthTexture=X}if(E.depthTexture&&!v.__autoAllocateDepthBuffer){if(U)throw new Error("target.depthTexture not supported in Cube render targets");Bt(v.__webglFramebuffer,E)}else if(U){v.__webglDepthbuffer=[];for(let X=0;X<6;X++)if(e.bindFramebuffer(n.FRAMEBUFFER,v.__webglFramebuffer[X]),v.__webglDepthbuffer[X]===void 0)v.__webglDepthbuffer[X]=n.createRenderbuffer(),ot(v.__webglDepthbuffer[X],E,!1);else{let tt=E.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,$=v.__webglDepthbuffer[X];n.bindRenderbuffer(n.RENDERBUFFER,$),n.framebufferRenderbuffer(n.FRAMEBUFFER,tt,n.RENDERBUFFER,$)}}else if(e.bindFramebuffer(n.FRAMEBUFFER,v.__webglFramebuffer),v.__webglDepthbuffer===void 0)v.__webglDepthbuffer=n.createRenderbuffer(),ot(v.__webglDepthbuffer,E,!1);else{let X=E.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,tt=v.__webglDepthbuffer;n.bindRenderbuffer(n.RENDERBUFFER,tt),n.framebufferRenderbuffer(n.FRAMEBUFFER,X,n.RENDERBUFFER,tt)}e.bindFramebuffer(n.FRAMEBUFFER,null)}function Vt(E,v,U){let X=i.get(E);v!==void 0&&St(X.__webglFramebuffer,E,E.texture,n.COLOR_ATTACHMENT0,n.TEXTURE_2D,0),U!==void 0&&qt(E)}function le(E){let v=E.texture,U=i.get(E),X=i.get(v);E.addEventListener("dispose",C);let tt=E.textures,$=E.isWebGLCubeRenderTarget===!0,Et=tt.length>1;if(Et||(X.__webglTexture===void 0&&(X.__webglTexture=n.createTexture()),X.__version=v.version,a.memory.textures++),$){U.__webglFramebuffer=[];for(let ut=0;ut<6;ut++)if(v.mipmaps&&v.mipmaps.length>0){U.__webglFramebuffer[ut]=[];for(let vt=0;vt<v.mipmaps.length;vt++)U.__webglFramebuffer[ut][vt]=n.createFramebuffer()}else U.__webglFramebuffer[ut]=n.createFramebuffer()}else{if(v.mipmaps&&v.mipmaps.length>0){U.__webglFramebuffer=[];for(let ut=0;ut<v.mipmaps.length;ut++)U.__webglFramebuffer[ut]=n.createFramebuffer()}else U.__webglFramebuffer=n.createFramebuffer();if(Et)for(let ut=0,vt=tt.length;ut<vt;ut++){let ne=i.get(tt[ut]);ne.__webglTexture===void 0&&(ne.__webglTexture=n.createTexture(),a.memory.textures++)}if(E.samples>0&&yt(E)===!1){U.__webglMultisampledFramebuffer=n.createFramebuffer(),U.__webglColorRenderbuffer=[],e.bindFramebuffer(n.FRAMEBUFFER,U.__webglMultisampledFramebuffer);for(let ut=0;ut<tt.length;ut++){let vt=tt[ut];U.__webglColorRenderbuffer[ut]=n.createRenderbuffer(),n.bindRenderbuffer(n.RENDERBUFFER,U.__webglColorRenderbuffer[ut]);let ne=r.convert(vt.format,vt.colorSpace),nt=r.convert(vt.type),_t=M(vt.internalFormat,ne,nt,vt.colorSpace,E.isXRRenderTarget===!0),Ht=et(E);n.renderbufferStorageMultisample(n.RENDERBUFFER,Ht,_t,E.width,E.height),n.framebufferRenderbuffer(n.FRAMEBUFFER,n.COLOR_ATTACHMENT0+ut,n.RENDERBUFFER,U.__webglColorRenderbuffer[ut])}n.bindRenderbuffer(n.RENDERBUFFER,null),E.depthBuffer&&(U.__webglDepthRenderbuffer=n.createRenderbuffer(),ot(U.__webglDepthRenderbuffer,E,!0)),e.bindFramebuffer(n.FRAMEBUFFER,null)}}if($){e.bindTexture(n.TEXTURE_CUBE_MAP,X.__webglTexture),Yt(n.TEXTURE_CUBE_MAP,v);for(let ut=0;ut<6;ut++)if(v.mipmaps&&v.mipmaps.length>0)for(let vt=0;vt<v.mipmaps.length;vt++)St(U.__webglFramebuffer[ut][vt],E,v,n.COLOR_ATTACHMENT0,n.TEXTURE_CUBE_MAP_POSITIVE_X+ut,vt);else St(U.__webglFramebuffer[ut],E,v,n.COLOR_ATTACHMENT0,n.TEXTURE_CUBE_MAP_POSITIVE_X+ut,0);m(v)&&p(n.TEXTURE_CUBE_MAP),e.unbindTexture()}else if(Et){for(let ut=0,vt=tt.length;ut<vt;ut++){let ne=tt[ut],nt=i.get(ne);e.bindTexture(n.TEXTURE_2D,nt.__webglTexture),Yt(n.TEXTURE_2D,ne),St(U.__webglFramebuffer,E,ne,n.COLOR_ATTACHMENT0+ut,n.TEXTURE_2D,0),m(ne)&&p(n.TEXTURE_2D)}e.unbindTexture()}else{let ut=n.TEXTURE_2D;if((E.isWebGL3DRenderTarget||E.isWebGLArrayRenderTarget)&&(ut=E.isWebGL3DRenderTarget?n.TEXTURE_3D:n.TEXTURE_2D_ARRAY),e.bindTexture(ut,X.__webglTexture),Yt(ut,v),v.mipmaps&&v.mipmaps.length>0)for(let vt=0;vt<v.mipmaps.length;vt++)St(U.__webglFramebuffer[vt],E,v,n.COLOR_ATTACHMENT0,ut,vt);else St(U.__webglFramebuffer,E,v,n.COLOR_ATTACHMENT0,ut,0);m(v)&&p(ut),e.unbindTexture()}E.depthBuffer&&qt(E)}function Q(E){let v=E.textures;for(let U=0,X=v.length;U<X;U++){let tt=v[U];if(m(tt)){let $=T(E),Et=i.get(tt).__webglTexture;e.bindTexture($,Et),p($),e.unbindTexture()}}}let st=[],A=[];function Ft(E){if(E.samples>0){if(yt(E)===!1){let v=E.textures,U=E.width,X=E.height,tt=n.COLOR_BUFFER_BIT,$=E.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,Et=i.get(E),ut=v.length>1;if(ut)for(let vt=0;vt<v.length;vt++)e.bindFramebuffer(n.FRAMEBUFFER,Et.__webglMultisampledFramebuffer),n.framebufferRenderbuffer(n.FRAMEBUFFER,n.COLOR_ATTACHMENT0+vt,n.RENDERBUFFER,null),e.bindFramebuffer(n.FRAMEBUFFER,Et.__webglFramebuffer),n.framebufferTexture2D(n.DRAW_FRAMEBUFFER,n.COLOR_ATTACHMENT0+vt,n.TEXTURE_2D,null,0);e.bindFramebuffer(n.READ_FRAMEBUFFER,Et.__webglMultisampledFramebuffer),e.bindFramebuffer(n.DRAW_FRAMEBUFFER,Et.__webglFramebuffer);for(let vt=0;vt<v.length;vt++){if(E.resolveDepthBuffer&&(E.depthBuffer&&(tt|=n.DEPTH_BUFFER_BIT),E.stencilBuffer&&E.resolveStencilBuffer&&(tt|=n.STENCIL_BUFFER_BIT)),ut){n.framebufferRenderbuffer(n.READ_FRAMEBUFFER,n.COLOR_ATTACHMENT0,n.RENDERBUFFER,Et.__webglColorRenderbuffer[vt]);let ne=i.get(v[vt]).__webglTexture;n.framebufferTexture2D(n.DRAW_FRAMEBUFFER,n.COLOR_ATTACHMENT0,n.TEXTURE_2D,ne,0)}n.blitFramebuffer(0,0,U,X,0,0,U,X,tt,n.NEAREST),l===!0&&(st.length=0,A.length=0,st.push(n.COLOR_ATTACHMENT0+vt),E.depthBuffer&&E.resolveDepthBuffer===!1&&(st.push($),A.push($),n.invalidateFramebuffer(n.DRAW_FRAMEBUFFER,A)),n.invalidateFramebuffer(n.READ_FRAMEBUFFER,st))}if(e.bindFramebuffer(n.READ_FRAMEBUFFER,null),e.bindFramebuffer(n.DRAW_FRAMEBUFFER,null),ut)for(let vt=0;vt<v.length;vt++){e.bindFramebuffer(n.FRAMEBUFFER,Et.__webglMultisampledFramebuffer),n.framebufferRenderbuffer(n.FRAMEBUFFER,n.COLOR_ATTACHMENT0+vt,n.RENDERBUFFER,Et.__webglColorRenderbuffer[vt]);let ne=i.get(v[vt]).__webglTexture;e.bindFramebuffer(n.FRAMEBUFFER,Et.__webglFramebuffer),n.framebufferTexture2D(n.DRAW_FRAMEBUFFER,n.COLOR_ATTACHMENT0+vt,n.TEXTURE_2D,ne,0)}e.bindFramebuffer(n.DRAW_FRAMEBUFFER,Et.__webglMultisampledFramebuffer)}else if(E.depthBuffer&&E.resolveDepthBuffer===!1&&l){let v=E.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT;n.invalidateFramebuffer(n.DRAW_FRAMEBUFFER,[v])}}}function et(E){return Math.min(s.maxSamples,E.samples)}function yt(E){let v=i.get(E);return E.samples>0&&t.has("WEBGL_multisampled_render_to_texture")===!0&&v.__useRenderToTexture!==!1}function lt(E){let v=a.render.frame;h.get(E)!==v&&(h.set(E,v),E.update())}function zt(E,v){let U=E.colorSpace,X=E.format,tt=E.type;return E.isCompressedTexture===!0||E.isVideoTexture===!0||U!==Lr&&U!==Yi&&(se.getTransfer(U)===fe?(X!==Hn||tt!==Ai)&&console.warn("THREE.WebGLTextures: sRGB encoded textures have to use RGBAFormat and UnsignedByteType."):console.error("THREE.WebGLTextures: Unsupported texture color space:",U)),v}function xt(E){return typeof HTMLImageElement<"u"&&E instanceof HTMLImageElement?(c.width=E.naturalWidth||E.width,c.height=E.naturalHeight||E.height):typeof VideoFrame<"u"&&E instanceof VideoFrame?(c.width=E.displayWidth,c.height=E.displayHeight):(c.width=E.width,c.height=E.height),c}this.allocateTextureUnit=N,this.resetTextureUnits=G,this.setTexture2D=J,this.setTexture2DArray=V,this.setTexture3D=it,this.setTextureCube=W,this.rebindTextures=Vt,this.setupRenderTarget=le,this.updateRenderTargetMipmap=Q,this.updateMultisampleRenderTarget=Ft,this.setupDepthRenderbuffer=qt,this.setupFrameBufferTexture=St,this.useMultisampledRTT=yt}function Z1(n,t){function e(i,s=Yi){let r,a=se.getTransfer(s);if(i===Ai)return n.UNSIGNED_BYTE;if(i===nu)return n.UNSIGNED_SHORT_4_4_4_4;if(i===iu)return n.UNSIGNED_SHORT_5_5_5_1;if(i===uf)return n.UNSIGNED_INT_5_9_9_9_REV;if(i===cf)return n.BYTE;if(i===hf)return n.SHORT;if(i===da)return n.UNSIGNED_SHORT;if(i===eu)return n.INT;if(i===Us)return n.UNSIGNED_INT;if(i===Kn)return n.FLOAT;if(i===en)return n.HALF_FLOAT;if(i===df)return n.ALPHA;if(i===ff)return n.RGB;if(i===Hn)return n.RGBA;if(i===pf)return n.LUMINANCE;if(i===mf)return n.LUMINANCE_ALPHA;if(i===Mr)return n.DEPTH_COMPONENT;if(i===Rr)return n.DEPTH_STENCIL;if(i===su)return n.RED;if(i===ru)return n.RED_INTEGER;if(i===gf)return n.RG;if(i===au)return n.RG_INTEGER;if(i===ou)return n.RGBA_INTEGER;if(i===To||i===Ao||i===Co||i===Ro)if(a===fe)if(r=t.get("WEBGL_compressed_texture_s3tc_srgb"),r!==null){if(i===To)return r.COMPRESSED_SRGB_S3TC_DXT1_EXT;if(i===Ao)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT;if(i===Co)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT;if(i===Ro)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT}else return null;else if(r=t.get("WEBGL_compressed_texture_s3tc"),r!==null){if(i===To)return r.COMPRESSED_RGB_S3TC_DXT1_EXT;if(i===Ao)return r.COMPRESSED_RGBA_S3TC_DXT1_EXT;if(i===Co)return r.COMPRESSED_RGBA_S3TC_DXT3_EXT;if(i===Ro)return r.COMPRESSED_RGBA_S3TC_DXT5_EXT}else return null;if(i===Dc||i===kc||i===Lc||i===Uc)if(r=t.get("WEBGL_compressed_texture_pvrtc"),r!==null){if(i===Dc)return r.COMPRESSED_RGB_PVRTC_4BPPV1_IMG;if(i===kc)return r.COMPRESSED_RGB_PVRTC_2BPPV1_IMG;if(i===Lc)return r.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG;if(i===Uc)return r.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG}else return null;if(i===Nc||i===Bc||i===Oc)if(r=t.get("WEBGL_compressed_texture_etc"),r!==null){if(i===Nc||i===Bc)return a===fe?r.COMPRESSED_SRGB8_ETC2:r.COMPRESSED_RGB8_ETC2;if(i===Oc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC:r.COMPRESSED_RGBA8_ETC2_EAC}else return null;if(i===zc||i===Hc||i===Vc||i===Gc||i===Wc||i===qc||i===Xc||i===$c||i===Yc||i===Zc||i===Jc||i===Kc||i===Qc||i===jc)if(r=t.get("WEBGL_compressed_texture_astc"),r!==null){if(i===zc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR:r.COMPRESSED_RGBA_ASTC_4x4_KHR;if(i===Hc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR:r.COMPRESSED_RGBA_ASTC_5x4_KHR;if(i===Vc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR:r.COMPRESSED_RGBA_ASTC_5x5_KHR;if(i===Gc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR:r.COMPRESSED_RGBA_ASTC_6x5_KHR;if(i===Wc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR:r.COMPRESSED_RGBA_ASTC_6x6_KHR;if(i===qc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR:r.COMPRESSED_RGBA_ASTC_8x5_KHR;if(i===Xc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR:r.COMPRESSED_RGBA_ASTC_8x6_KHR;if(i===$c)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR:r.COMPRESSED_RGBA_ASTC_8x8_KHR;if(i===Yc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR:r.COMPRESSED_RGBA_ASTC_10x5_KHR;if(i===Zc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR:r.COMPRESSED_RGBA_ASTC_10x6_KHR;if(i===Jc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR:r.COMPRESSED_RGBA_ASTC_10x8_KHR;if(i===Kc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR:r.COMPRESSED_RGBA_ASTC_10x10_KHR;if(i===Qc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR:r.COMPRESSED_RGBA_ASTC_12x10_KHR;if(i===jc)return a===fe?r.COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR:r.COMPRESSED_RGBA_ASTC_12x12_KHR}else return null;if(i===Po||i===th||i===eh)if(r=t.get("EXT_texture_compression_bptc"),r!==null){if(i===Po)return a===fe?r.COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT:r.COMPRESSED_RGBA_BPTC_UNORM_EXT;if(i===th)return r.COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT;if(i===eh)return r.COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT}else return null;if(i===xf||i===nh||i===ih||i===sh)if(r=t.get("EXT_texture_compression_rgtc"),r!==null){if(i===Po)return r.COMPRESSED_RED_RGTC1_EXT;if(i===nh)return r.COMPRESSED_SIGNED_RED_RGTC1_EXT;if(i===ih)return r.COMPRESSED_RED_GREEN_RGTC2_EXT;if(i===sh)return r.COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT}else return null;return i===Cr?n.UNSIGNED_INT_24_8:n[i]!==void 0?n[i]:null}return{convert:e}}var Mh=class extends Qe{constructor(t=[]){super(),this.isArrayCamera=!0,this.cameras=t}},He=class extends ln{constructor(){super(),this.isGroup=!0,this.type="Group"}},J1={type:"move"},la=class{constructor(){this._targetRay=null,this._grip=null,this._hand=null}getHandSpace(){return this._hand===null&&(this._hand=new He,this._hand.matrixAutoUpdate=!1,this._hand.visible=!1,this._hand.joints={},this._hand.inputState={pinching:!1}),this._hand}getTargetRaySpace(){return this._targetRay===null&&(this._targetRay=new He,this._targetRay.matrixAutoUpdate=!1,this._targetRay.visible=!1,this._targetRay.hasLinearVelocity=!1,this._targetRay.linearVelocity=new I,this._targetRay.hasAngularVelocity=!1,this._targetRay.angularVelocity=new I),this._targetRay}getGripSpace(){return this._grip===null&&(this._grip=new He,this._grip.matrixAutoUpdate=!1,this._grip.visible=!1,this._grip.hasLinearVelocity=!1,this._grip.linearVelocity=new I,this._grip.hasAngularVelocity=!1,this._grip.angularVelocity=new I),this._grip}dispatchEvent(t){return this._targetRay!==null&&this._targetRay.dispatchEvent(t),this._grip!==null&&this._grip.dispatchEvent(t),this._hand!==null&&this._hand.dispatchEvent(t),this}connect(t){if(t&&t.hand){let e=this._hand;if(e)for(let i of t.hand.values())this._getHandJoint(e,i)}return this.dispatchEvent({type:"connected",data:t}),this}disconnect(t){return this.dispatchEvent({type:"disconnected",data:t}),this._targetRay!==null&&(this._targetRay.visible=!1),this._grip!==null&&(this._grip.visible=!1),this._hand!==null&&(this._hand.visible=!1),this}update(t,e,i){let s=null,r=null,a=null,o=this._targetRay,l=this._grip,c=this._hand;if(t&&e.session.visibilityState!=="visible-blurred"){if(c&&t.hand){a=!0;for(let x of t.hand.values()){let m=e.getJointPose(x,i),p=this._getHandJoint(c,x);m!==null&&(p.matrix.fromArray(m.transform.matrix),p.matrix.decompose(p.position,p.rotation,p.scale),p.matrixWorldNeedsUpdate=!0,p.jointRadius=m.radius),p.visible=m!==null}let h=c.joints["index-finger-tip"],u=c.joints["thumb-tip"],d=h.position.distanceTo(u.position),f=.02,g=.005;c.inputState.pinching&&d>f+g?(c.inputState.pinching=!1,this.dispatchEvent({type:"pinchend",handedness:t.handedness,target:this})):!c.inputState.pinching&&d<=f-g&&(c.inputState.pinching=!0,this.dispatchEvent({type:"pinchstart",handedness:t.handedness,target:this}))}else l!==null&&t.gripSpace&&(r=e.getPose(t.gripSpace,i),r!==null&&(l.matrix.fromArray(r.transform.matrix),l.matrix.decompose(l.position,l.rotation,l.scale),l.matrixWorldNeedsUpdate=!0,r.linearVelocity?(l.hasLinearVelocity=!0,l.linearVelocity.copy(r.linearVelocity)):l.hasLinearVelocity=!1,r.angularVelocity?(l.hasAngularVelocity=!0,l.angularVelocity.copy(r.angularVelocity)):l.hasAngularVelocity=!1));o!==null&&(s=e.getPose(t.targetRaySpace,i),s===null&&r!==null&&(s=r),s!==null&&(o.matrix.fromArray(s.transform.matrix),o.matrix.decompose(o.position,o.rotation,o.scale),o.matrixWorldNeedsUpdate=!0,s.linearVelocity?(o.hasLinearVelocity=!0,o.linearVelocity.copy(s.linearVelocity)):o.hasLinearVelocity=!1,s.angularVelocity?(o.hasAngularVelocity=!0,o.angularVelocity.copy(s.angularVelocity)):o.hasAngularVelocity=!1,this.dispatchEvent(J1)))}return o!==null&&(o.visible=s!==null),l!==null&&(l.visible=r!==null),c!==null&&(c.visible=a!==null),this}_getHandJoint(t,e){if(t.joints[e.jointName]===void 0){let i=new He;i.matrixAutoUpdate=!1,i.visible=!1,t.joints[e.jointName]=i,t.add(i)}return t.joints[e.jointName]}},K1=`
void main() {

	gl_Position = vec4( position, 1.0 );

}`,Q1=`
uniform sampler2DArray depthColor;
uniform float depthWidth;
uniform float depthHeight;

void main() {

	vec2 coord = vec2( gl_FragCoord.x / depthWidth, gl_FragCoord.y / depthHeight );

	if ( coord.x >= 1.0 ) {

		gl_FragDepth = texture( depthColor, vec3( coord.x - 1.0, coord.y, 1 ) ).r;

	} else {

		gl_FragDepth = texture( depthColor, vec3( coord.x, coord.y, 0 ) ).r;

	}

}`,Sh=class{constructor(){this.texture=null,this.mesh=null,this.depthNear=0,this.depthFar=0}init(t,e,i){if(this.texture===null){let s=new on,r=t.properties.get(s);r.__webglTexture=e.texture,(e.depthNear!=i.depthNear||e.depthFar!=i.depthFar)&&(this.depthNear=e.depthNear,this.depthFar=e.depthFar),this.texture=s}}getMesh(t){if(this.texture!==null&&this.mesh===null){let e=t.cameras[0].viewport,i=new Te({vertexShader:K1,fragmentShader:Q1,uniforms:{depthColor:{value:this.texture},depthWidth:{value:e.z},depthHeight:{value:e.w}}});this.mesh=new Ot(new ei(20,20),i)}return this.mesh}reset(){this.texture=null,this.mesh=null}getDepthTexture(){return this.texture}},wh=class extends ji{constructor(t,e){super();let i=this,s=null,r=1,a=null,o="local-floor",l=1,c=null,h=null,u=null,d=null,f=null,g=null,x=new Sh,m=e.getContextAttributes(),p=null,T=null,M=[],y=[],D=new j,S=null,C=new Qe;C.viewport=new xe;let P=new Qe;P.viewport=new xe;let w=[C,P],_=new Mh,R=null,G=null;this.cameraAutoUpdate=!0,this.enabled=!1,this.isPresenting=!1,this.getController=function(Y){let rt=M[Y];return rt===void 0&&(rt=new la,M[Y]=rt),rt.getTargetRaySpace()},this.getControllerGrip=function(Y){let rt=M[Y];return rt===void 0&&(rt=new la,M[Y]=rt),rt.getGripSpace()},this.getHand=function(Y){let rt=M[Y];return rt===void 0&&(rt=new la,M[Y]=rt),rt.getHandSpace()};function N(Y){let rt=y.indexOf(Y.inputSource);if(rt===-1)return;let St=M[rt];St!==void 0&&(St.update(Y.inputSource,Y.frame,c||a),St.dispatchEvent({type:Y.type,data:Y.inputSource}))}function z(){s.removeEventListener("select",N),s.removeEventListener("selectstart",N),s.removeEventListener("selectend",N),s.removeEventListener("squeeze",N),s.removeEventListener("squeezestart",N),s.removeEventListener("squeezeend",N),s.removeEventListener("end",z),s.removeEventListener("inputsourceschange",J);for(let Y=0;Y<M.length;Y++){let rt=y[Y];rt!==null&&(y[Y]=null,M[Y].disconnect(rt))}R=null,G=null,x.reset(),t.setRenderTarget(p),f=null,d=null,u=null,s=null,T=null,he.stop(),i.isPresenting=!1,t.setPixelRatio(S),t.setSize(D.width,D.height,!1),i.dispatchEvent({type:"sessionend"})}this.setFramebufferScaleFactor=function(Y){r=Y,i.isPresenting===!0&&console.warn("THREE.WebXRManager: Cannot change framebuffer scale while presenting.")},this.setReferenceSpaceType=function(Y){o=Y,i.isPresenting===!0&&console.warn("THREE.WebXRManager: Cannot change reference space type while presenting.")},this.getReferenceSpace=function(){return c||a},this.setReferenceSpace=function(Y){c=Y},this.getBaseLayer=function(){return d!==null?d:f},this.getBinding=function(){return u},this.getFrame=function(){return g},this.getSession=function(){return s},this.setSession=async function(Y){if(s=Y,s!==null){if(p=t.getRenderTarget(),s.addEventListener("select",N),s.addEventListener("selectstart",N),s.addEventListener("selectend",N),s.addEventListener("squeeze",N),s.addEventListener("squeezestart",N),s.addEventListener("squeezeend",N),s.addEventListener("end",z),s.addEventListener("inputsourceschange",J),m.xrCompatible!==!0&&await e.makeXRCompatible(),S=t.getPixelRatio(),t.getSize(D),s.renderState.layers===void 0){let rt={antialias:m.antialias,alpha:!0,depth:m.depth,stencil:m.stencil,framebufferScaleFactor:r};f=new XRWebGLLayer(s,e,rt),s.updateRenderState({baseLayer:f}),t.setPixelRatio(1),t.setSize(f.framebufferWidth,f.framebufferHeight,!1),T=new Ue(f.framebufferWidth,f.framebufferHeight,{format:Hn,type:Ai,colorSpace:t.outputColorSpace,stencilBuffer:m.stencil})}else{let rt=null,St=null,ot=null;m.depth&&(ot=m.stencil?e.DEPTH24_STENCIL8:e.DEPTH_COMPONENT24,rt=m.stencil?Rr:Mr,St=m.stencil?Cr:Us);let Bt={colorFormat:e.RGBA8,depthFormat:ot,scaleFactor:r};u=new XRWebGLBinding(s,e),d=u.createProjectionLayer(Bt),s.updateRenderState({layers:[d]}),t.setPixelRatio(1),t.setSize(d.textureWidth,d.textureHeight,!1),T=new Ue(d.textureWidth,d.textureHeight,{format:Hn,type:Ai,depthTexture:new zo(d.textureWidth,d.textureHeight,St,void 0,void 0,void 0,void 0,void 0,void 0,rt),stencilBuffer:m.stencil,colorSpace:t.outputColorSpace,samples:m.antialias?4:0,resolveDepthBuffer:d.ignoreDepthValues===!1})}T.isXRRenderTarget=!0,this.setFoveation(l),c=null,a=await s.requestReferenceSpace(o),he.setContext(s),he.start(),i.isPresenting=!0,i.dispatchEvent({type:"sessionstart"})}},this.getEnvironmentBlendMode=function(){if(s!==null)return s.environmentBlendMode},this.getDepthTexture=function(){return x.getDepthTexture()};function J(Y){for(let rt=0;rt<Y.removed.length;rt++){let St=Y.removed[rt],ot=y.indexOf(St);ot>=0&&(y[ot]=null,M[ot].disconnect(St))}for(let rt=0;rt<Y.added.length;rt++){let St=Y.added[rt],ot=y.indexOf(St);if(ot===-1){for(let qt=0;qt<M.length;qt++)if(qt>=y.length){y.push(St),ot=qt;break}else if(y[qt]===null){y[qt]=St,ot=qt;break}if(ot===-1)break}let Bt=M[ot];Bt&&Bt.connect(St)}}let V=new I,it=new I;function W(Y,rt,St){V.setFromMatrixPosition(rt.matrixWorld),it.setFromMatrixPosition(St.matrixWorld);let ot=V.distanceTo(it),Bt=rt.projectionMatrix.elements,qt=St.projectionMatrix.elements,Vt=Bt[14]/(Bt[10]-1),le=Bt[14]/(Bt[10]+1),Q=(Bt[9]+1)/Bt[5],st=(Bt[9]-1)/Bt[5],A=(Bt[8]-1)/Bt[0],Ft=(qt[8]+1)/qt[0],et=Vt*A,yt=Vt*Ft,lt=ot/(-A+Ft),zt=lt*-A;if(rt.matrixWorld.decompose(Y.position,Y.quaternion,Y.scale),Y.translateX(zt),Y.translateZ(lt),Y.matrixWorld.compose(Y.position,Y.quaternion,Y.scale),Y.matrixWorldInverse.copy(Y.matrixWorld).invert(),Bt[10]===-1)Y.projectionMatrix.copy(rt.projectionMatrix),Y.projectionMatrixInverse.copy(rt.projectionMatrixInverse);else{let xt=Vt+lt,E=le+lt,v=et-zt,U=yt+(ot-zt),X=Q*le/E*xt,tt=st*le/E*xt;Y.projectionMatrix.makePerspective(v,U,X,tt,xt,E),Y.projectionMatrixInverse.copy(Y.projectionMatrix).invert()}}function ct(Y,rt){rt===null?Y.matrixWorld.copy(Y.matrix):Y.matrixWorld.multiplyMatrices(rt.matrixWorld,Y.matrix),Y.matrixWorldInverse.copy(Y.matrixWorld).invert()}this.updateCamera=function(Y){if(s===null)return;let rt=Y.near,St=Y.far;x.texture!==null&&(x.depthNear>0&&(rt=x.depthNear),x.depthFar>0&&(St=x.depthFar)),_.near=P.near=C.near=rt,_.far=P.far=C.far=St,(R!==_.near||G!==_.far)&&(s.updateRenderState({depthNear:_.near,depthFar:_.far}),R=_.near,G=_.far),C.layers.mask=Y.layers.mask|2,P.layers.mask=Y.layers.mask|4,_.layers.mask=C.layers.mask|P.layers.mask;let ot=Y.parent,Bt=_.cameras;ct(_,ot);for(let qt=0;qt<Bt.length;qt++)ct(Bt[qt],ot);Bt.length===2?W(_,C,P):_.projectionMatrix.copy(C.projectionMatrix),gt(Y,_,ot)};function gt(Y,rt,St){St===null?Y.matrix.copy(rt.matrixWorld):(Y.matrix.copy(St.matrixWorld),Y.matrix.invert(),Y.matrix.multiply(rt.matrixWorld)),Y.matrix.decompose(Y.position,Y.quaternion,Y.scale),Y.updateMatrixWorld(!0),Y.projectionMatrix.copy(rt.projectionMatrix),Y.projectionMatrixInverse.copy(rt.projectionMatrixInverse),Y.isPerspectiveCamera&&(Y.fov=oh*2*Math.atan(1/Y.projectionMatrix.elements[5]),Y.zoom=1)}this.getCamera=function(){return _},this.getFoveation=function(){if(!(d===null&&f===null))return l},this.setFoveation=function(Y){l=Y,d!==null&&(d.fixedFoveation=Y),f!==null&&f.fixedFoveation!==void 0&&(f.fixedFoveation=Y)},this.hasDepthSensing=function(){return x.texture!==null},this.getDepthSensingMesh=function(){return x.getMesh(_)};let Mt=null;function Yt(Y,rt){if(h=rt.getViewerPose(c||a),g=rt,h!==null){let St=h.views;f!==null&&(t.setRenderTargetFramebuffer(T,f.framebuffer),t.setRenderTarget(T));let ot=!1;St.length!==_.cameras.length&&(_.cameras.length=0,ot=!0);for(let qt=0;qt<St.length;qt++){let Vt=St[qt],le=null;if(f!==null)le=f.getViewport(Vt);else{let st=u.getViewSubImage(d,Vt);le=st.viewport,qt===0&&(t.setRenderTargetTextures(T,st.colorTexture,d.ignoreDepthValues?void 0:st.depthStencilTexture),t.setRenderTarget(T))}let Q=w[qt];Q===void 0&&(Q=new Qe,Q.layers.enable(qt),Q.viewport=new xe,w[qt]=Q),Q.matrix.fromArray(Vt.transform.matrix),Q.matrix.decompose(Q.position,Q.quaternion,Q.scale),Q.projectionMatrix.fromArray(Vt.projectionMatrix),Q.projectionMatrixInverse.copy(Q.projectionMatrix).invert(),Q.viewport.set(le.x,le.y,le.width,le.height),qt===0&&(_.matrix.copy(Q.matrix),_.matrix.decompose(_.position,_.quaternion,_.scale)),ot===!0&&_.cameras.push(Q)}let Bt=s.enabledFeatures;if(Bt&&Bt.includes("depth-sensing")){let qt=u.getDepthInformation(St[0]);qt&&qt.isValid&&qt.texture&&x.init(t,qt,s.renderState)}}for(let St=0;St<M.length;St++){let ot=y[St],Bt=M[St];ot!==null&&Bt!==void 0&&Bt.update(ot,rt,c||a)}Mt&&Mt(Y,rt),rt.detectedPlanes&&i.dispatchEvent({type:"planesdetected",data:rt}),g=null}let he=new Sf;he.setAnimationLoop(Yt),this.setAnimationLoop=function(Y){Mt=Y},this.dispose=function(){}}},Is=new ti,j1=new pe;function tv(n,t){function e(m,p){m.matrixAutoUpdate===!0&&m.updateMatrix(),p.value.copy(m.matrix)}function i(m,p){p.color.getRGB(m.fogColor.value,Mf(n)),p.isFog?(m.fogNear.value=p.near,m.fogFar.value=p.far):p.isFogExp2&&(m.fogDensity.value=p.density)}function s(m,p,T,M,y){p.isMeshBasicMaterial||p.isMeshLambertMaterial?r(m,p):p.isMeshToonMaterial?(r(m,p),u(m,p)):p.isMeshPhongMaterial?(r(m,p),h(m,p)):p.isMeshStandardMaterial?(r(m,p),d(m,p),p.isMeshPhysicalMaterial&&f(m,p,y)):p.isMeshMatcapMaterial?(r(m,p),g(m,p)):p.isMeshDepthMaterial?r(m,p):p.isMeshDistanceMaterial?(r(m,p),x(m,p)):p.isMeshNormalMaterial?r(m,p):p.isLineBasicMaterial?(a(m,p),p.isLineDashedMaterial&&o(m,p)):p.isPointsMaterial?l(m,p,T,M):p.isSpriteMaterial?c(m,p):p.isShadowMaterial?(m.color.value.copy(p.color),m.opacity.value=p.opacity):p.isShaderMaterial&&(p.uniformsNeedUpdate=!1)}function r(m,p){m.opacity.value=p.opacity,p.color&&m.diffuse.value.copy(p.color),p.emissive&&m.emissive.value.copy(p.emissive).multiplyScalar(p.emissiveIntensity),p.map&&(m.map.value=p.map,e(p.map,m.mapTransform)),p.alphaMap&&(m.alphaMap.value=p.alphaMap,e(p.alphaMap,m.alphaMapTransform)),p.bumpMap&&(m.bumpMap.value=p.bumpMap,e(p.bumpMap,m.bumpMapTransform),m.bumpScale.value=p.bumpScale,p.side===je&&(m.bumpScale.value*=-1)),p.normalMap&&(m.normalMap.value=p.normalMap,e(p.normalMap,m.normalMapTransform),m.normalScale.value.copy(p.normalScale),p.side===je&&m.normalScale.value.negate()),p.displacementMap&&(m.displacementMap.value=p.displacementMap,e(p.displacementMap,m.displacementMapTransform),m.displacementScale.value=p.displacementScale,m.displacementBias.value=p.displacementBias),p.emissiveMap&&(m.emissiveMap.value=p.emissiveMap,e(p.emissiveMap,m.emissiveMapTransform)),p.specularMap&&(m.specularMap.value=p.specularMap,e(p.specularMap,m.specularMapTransform)),p.alphaTest>0&&(m.alphaTest.value=p.alphaTest);let T=t.get(p),M=T.envMap,y=T.envMapRotation;M&&(m.envMap.value=M,Is.copy(y),Is.x*=-1,Is.y*=-1,Is.z*=-1,M.isCubeTexture&&M.isRenderTargetTexture===!1&&(Is.y*=-1,Is.z*=-1),m.envMapRotation.value.setFromMatrix4(j1.makeRotationFromEuler(Is)),m.flipEnvMap.value=M.isCubeTexture&&M.isRenderTargetTexture===!1?-1:1,m.reflectivity.value=p.reflectivity,m.ior.value=p.ior,m.refractionRatio.value=p.refractionRatio),p.lightMap&&(m.lightMap.value=p.lightMap,m.lightMapIntensity.value=p.lightMapIntensity,e(p.lightMap,m.lightMapTransform)),p.aoMap&&(m.aoMap.value=p.aoMap,m.aoMapIntensity.value=p.aoMapIntensity,e(p.aoMap,m.aoMapTransform))}function a(m,p){m.diffuse.value.copy(p.color),m.opacity.value=p.opacity,p.map&&(m.map.value=p.map,e(p.map,m.mapTransform))}function o(m,p){m.dashSize.value=p.dashSize,m.totalSize.value=p.dashSize+p.gapSize,m.scale.value=p.scale}function l(m,p,T,M){m.diffuse.value.copy(p.color),m.opacity.value=p.opacity,m.size.value=p.size*T,m.scale.value=M*.5,p.map&&(m.map.value=p.map,e(p.map,m.uvTransform)),p.alphaMap&&(m.alphaMap.value=p.alphaMap,e(p.alphaMap,m.alphaMapTransform)),p.alphaTest>0&&(m.alphaTest.value=p.alphaTest)}function c(m,p){m.diffuse.value.copy(p.color),m.opacity.value=p.opacity,m.rotation.value=p.rotation,p.map&&(m.map.value=p.map,e(p.map,m.mapTransform)),p.alphaMap&&(m.alphaMap.value=p.alphaMap,e(p.alphaMap,m.alphaMapTransform)),p.alphaTest>0&&(m.alphaTest.value=p.alphaTest)}function h(m,p){m.specular.value.copy(p.specular),m.shininess.value=Math.max(p.shininess,1e-4)}function u(m,p){p.gradientMap&&(m.gradientMap.value=p.gradientMap)}function d(m,p){m.metalness.value=p.metalness,p.metalnessMap&&(m.metalnessMap.value=p.metalnessMap,e(p.metalnessMap,m.metalnessMapTransform)),m.roughness.value=p.roughness,p.roughnessMap&&(m.roughnessMap.value=p.roughnessMap,e(p.roughnessMap,m.roughnessMapTransform)),p.envMap&&(m.envMapIntensity.value=p.envMapIntensity)}function f(m,p,T){m.ior.value=p.ior,p.sheen>0&&(m.sheenColor.value.copy(p.sheenColor).multiplyScalar(p.sheen),m.sheenRoughness.value=p.sheenRoughness,p.sheenColorMap&&(m.sheenColorMap.value=p.sheenColorMap,e(p.sheenColorMap,m.sheenColorMapTransform)),p.sheenRoughnessMap&&(m.sheenRoughnessMap.value=p.sheenRoughnessMap,e(p.sheenRoughnessMap,m.sheenRoughnessMapTransform))),p.clearcoat>0&&(m.clearcoat.value=p.clearcoat,m.clearcoatRoughness.value=p.clearcoatRoughness,p.clearcoatMap&&(m.clearcoatMap.value=p.clearcoatMap,e(p.clearcoatMap,m.clearcoatMapTransform)),p.clearcoatRoughnessMap&&(m.clearcoatRoughnessMap.value=p.clearcoatRoughnessMap,e(p.clearcoatRoughnessMap,m.clearcoatRoughnessMapTransform)),p.clearcoatNormalMap&&(m.clearcoatNormalMap.value=p.clearcoatNormalMap,e(p.clearcoatNormalMap,m.clearcoatNormalMapTransform),m.clearcoatNormalScale.value.copy(p.clearcoatNormalScale),p.side===je&&m.clearcoatNormalScale.value.negate())),p.dispersion>0&&(m.dispersion.value=p.dispersion),p.iridescence>0&&(m.iridescence.value=p.iridescence,m.iridescenceIOR.value=p.iridescenceIOR,m.iridescenceThicknessMinimum.value=p.iridescenceThicknessRange[0],m.iridescenceThicknessMaximum.value=p.iridescenceThicknessRange[1],p.iridescenceMap&&(m.iridescenceMap.value=p.iridescenceMap,e(p.iridescenceMap,m.iridescenceMapTransform)),p.iridescenceThicknessMap&&(m.iridescenceThicknessMap.value=p.iridescenceThicknessMap,e(p.iridescenceThicknessMap,m.iridescenceThicknessMapTransform))),p.transmission>0&&(m.transmission.value=p.transmission,m.transmissionSamplerMap.value=T.texture,m.transmissionSamplerSize.value.set(T.width,T.height),p.transmissionMap&&(m.transmissionMap.value=p.transmissionMap,e(p.transmissionMap,m.transmissionMapTransform)),m.thickness.value=p.thickness,p.thicknessMap&&(m.thicknessMap.value=p.thicknessMap,e(p.thicknessMap,m.thicknessMapTransform)),m.attenuationDistance.value=p.attenuationDistance,m.attenuationColor.value.copy(p.attenuationColor)),p.anisotropy>0&&(m.anisotropyVector.value.set(p.anisotropy*Math.cos(p.anisotropyRotation),p.anisotropy*Math.sin(p.anisotropyRotation)),p.anisotropyMap&&(m.anisotropyMap.value=p.anisotropyMap,e(p.anisotropyMap,m.anisotropyMapTransform))),m.specularIntensity.value=p.specularIntensity,m.specularColor.value.copy(p.specularColor),p.specularColorMap&&(m.specularColorMap.value=p.specularColorMap,e(p.specularColorMap,m.specularColorMapTransform)),p.specularIntensityMap&&(m.specularIntensityMap.value=p.specularIntensityMap,e(p.specularIntensityMap,m.specularIntensityMapTransform))}function g(m,p){p.matcap&&(m.matcap.value=p.matcap)}function x(m,p){let T=t.get(p).light;m.referencePosition.value.setFromMatrixPosition(T.matrixWorld),m.nearDistance.value=T.shadow.camera.near,m.farDistance.value=T.shadow.camera.far}return{refreshFogUniforms:i,refreshMaterialUniforms:s}}function ev(n,t,e,i){let s={},r={},a=[],o=n.getParameter(n.MAX_UNIFORM_BUFFER_BINDINGS);function l(T,M){let y=M.program;i.uniformBlockBinding(T,y)}function c(T,M){let y=s[T.id];y===void 0&&(g(T),y=h(T),s[T.id]=y,T.addEventListener("dispose",m));let D=M.program;i.updateUBOMapping(T,D);let S=t.render.frame;r[T.id]!==S&&(d(T),r[T.id]=S)}function h(T){let M=u();T.__bindingPointIndex=M;let y=n.createBuffer(),D=T.__size,S=T.usage;return n.bindBuffer(n.UNIFORM_BUFFER,y),n.bufferData(n.UNIFORM_BUFFER,D,S),n.bindBuffer(n.UNIFORM_BUFFER,null),n.bindBufferBase(n.UNIFORM_BUFFER,M,y),y}function u(){for(let T=0;T<o;T++)if(a.indexOf(T)===-1)return a.push(T),T;return console.error("THREE.WebGLRenderer: Maximum number of simultaneously usable uniforms groups reached."),0}function d(T){let M=s[T.id],y=T.uniforms,D=T.__cache;n.bindBuffer(n.UNIFORM_BUFFER,M);for(let S=0,C=y.length;S<C;S++){let P=Array.isArray(y[S])?y[S]:[y[S]];for(let w=0,_=P.length;w<_;w++){let R=P[w];if(f(R,S,w,D)===!0){let G=R.__offset,N=Array.isArray(R.value)?R.value:[R.value],z=0;for(let J=0;J<N.length;J++){let V=N[J],it=x(V);typeof V=="number"||typeof V=="boolean"?(R.__data[0]=V,n.bufferSubData(n.UNIFORM_BUFFER,G+z,R.__data)):V.isMatrix3?(R.__data[0]=V.elements[0],R.__data[1]=V.elements[1],R.__data[2]=V.elements[2],R.__data[3]=0,R.__data[4]=V.elements[3],R.__data[5]=V.elements[4],R.__data[6]=V.elements[5],R.__data[7]=0,R.__data[8]=V.elements[6],R.__data[9]=V.elements[7],R.__data[10]=V.elements[8],R.__data[11]=0):(V.toArray(R.__data,z),z+=it.storage/Float32Array.BYTES_PER_ELEMENT)}n.bufferSubData(n.UNIFORM_BUFFER,G,R.__data)}}}n.bindBuffer(n.UNIFORM_BUFFER,null)}function f(T,M,y,D){let S=T.value,C=M+"_"+y;if(D[C]===void 0)return typeof S=="number"||typeof S=="boolean"?D[C]=S:D[C]=S.clone(),!0;{let P=D[C];if(typeof S=="number"||typeof S=="boolean"){if(P!==S)return D[C]=S,!0}else if(P.equals(S)===!1)return P.copy(S),!0}return!1}function g(T){let M=T.uniforms,y=0,D=16;for(let C=0,P=M.length;C<P;C++){let w=Array.isArray(M[C])?M[C]:[M[C]];for(let _=0,R=w.length;_<R;_++){let G=w[_],N=Array.isArray(G.value)?G.value:[G.value];for(let z=0,J=N.length;z<J;z++){let V=N[z],it=x(V),W=y%D,ct=W%it.boundary,gt=W+ct;y+=ct,gt!==0&&D-gt<it.storage&&(y+=D-gt),G.__data=new Float32Array(it.storage/Float32Array.BYTES_PER_ELEMENT),G.__offset=y,y+=it.storage}}}let S=y%D;return S>0&&(y+=D-S),T.__size=y,T.__cache={},this}function x(T){let M={boundary:0,storage:0};return typeof T=="number"||typeof T=="boolean"?(M.boundary=4,M.storage=4):T.isVector2?(M.boundary=8,M.storage=8):T.isVector3||T.isColor?(M.boundary=16,M.storage=12):T.isVector4?(M.boundary=16,M.storage=16):T.isMatrix3?(M.boundary=48,M.storage=48):T.isMatrix4?(M.boundary=64,M.storage=64):T.isTexture?console.warn("THREE.WebGLRenderer: Texture samplers can not be part of an uniforms group."):console.warn("THREE.WebGLRenderer: Unsupported uniform value type.",T),M}function m(T){let M=T.target;M.removeEventListener("dispose",m);let y=a.indexOf(M.__bindingPointIndex);a.splice(y,1),n.deleteBuffer(s[M.id]),delete s[M.id],delete r[M.id]}function p(){for(let T in s)n.deleteBuffer(s[T]);a=[],s={},r={}}return{bind:l,update:c,dispose:p}}var Ho=class{constructor(t={}){let{canvas:e=am(),context:i=null,depth:s=!0,stencil:r=!1,alpha:a=!1,antialias:o=!1,premultipliedAlpha:l=!0,preserveDrawingBuffer:c=!1,powerPreference:h="default",failIfMajorPerformanceCaveat:u=!1,reverseDepthBuffer:d=!1}=t;this.isWebGLRenderer=!0;let f;if(i!==null){if(typeof WebGLRenderingContext<"u"&&i instanceof WebGLRenderingContext)throw new Error("THREE.WebGLRenderer: WebGL 1 is not supported since r163.");f=i.getContextAttributes().alpha}else f=a;let g=new Uint32Array(4),x=new Int32Array(4),m=null,p=null,T=[],M=[];this.domElement=e,this.debug={checkShaderErrors:!0,onShaderError:null},this.autoClear=!0,this.autoClearColor=!0,this.autoClearDepth=!0,this.autoClearStencil=!0,this.sortObjects=!0,this.clippingPlanes=[],this.localClippingEnabled=!1,this._outputColorSpace=Ke,this.toneMapping=Ji,this.toneMappingExposure=1;let y=this,D=!1,S=0,C=0,P=null,w=-1,_=null,R=new xe,G=new xe,N=null,z=new Pt(0),J=0,V=e.width,it=e.height,W=1,ct=null,gt=null,Mt=new xe(0,0,V,it),Yt=new xe(0,0,V,it),he=!1,Y=new pa,rt=!1,St=!1,ot=new pe,Bt=new pe,qt=new I,Vt=new xe,le={background:null,fog:null,environment:null,overrideMaterial:null,isScene:!0},Q=!1;function st(){return P===null?W:1}let A=i;function Ft(b,k){return e.getContext(b,k)}try{let b={alpha:!0,depth:s,stencil:r,antialias:o,premultipliedAlpha:l,preserveDrawingBuffer:c,powerPreference:h,failIfMajorPerformanceCaveat:u};if("setAttribute"in e&&e.setAttribute("data-engine","three.js r170"),e.addEventListener("webglcontextlost",K,!1),e.addEventListener("webglcontextrestored",mt,!1),e.addEventListener("webglcontextcreationerror",ft,!1),A===null){let k="webgl2";if(A=Ft(k,b),A===null)throw Ft(k)?new Error("Error creating WebGL context with your selected attributes."):new Error("Error creating WebGL context.")}}catch(b){throw console.error("THREE.WebGLRenderer: "+b.message),b}let et,yt,lt,zt,xt,E,v,U,X,tt,$,Et,ut,vt,ne,nt,_t,Ht,Wt,bt,ae,jt,ve,F;function dt(){et=new xx(A),et.init(),jt=new Z1(A,et),yt=new ux(A,et,t,jt),lt=new X1(A,et),yt.reverseDepthBuffer&&d&&lt.buffers.depth.setReversed(!0),zt=new _x(A),xt=new D1,E=new Y1(A,et,lt,xt,yt,jt,zt),v=new fx(y),U=new gx(y),X=new Am(A),ve=new cx(A,X),tt=new vx(A,X,zt,ve),$=new Mx(A,tt,X,zt),Wt=new bx(A,yt,E),nt=new dx(xt),Et=new F1(y,v,U,et,yt,ve,nt),ut=new tv(y,xt),vt=new L1,ne=new H1(et),Ht=new lx(y,v,U,lt,$,f,l),_t=new W1(y,$,yt),F=new ev(A,zt,yt,lt),bt=new hx(A,et,zt),ae=new yx(A,et,zt),zt.programs=Et.programs,y.capabilities=yt,y.extensions=et,y.properties=xt,y.renderLists=vt,y.shadowMap=_t,y.state=lt,y.info=zt}dt();let q=new wh(y,A);this.xr=q,this.getContext=function(){return A},this.getContextAttributes=function(){return A.getContextAttributes()},this.forceContextLoss=function(){let b=et.get("WEBGL_lose_context");b&&b.loseContext()},this.forceContextRestore=function(){let b=et.get("WEBGL_lose_context");b&&b.restoreContext()},this.getPixelRatio=function(){return W},this.setPixelRatio=function(b){b!==void 0&&(W=b,this.setSize(V,it,!1))},this.getSize=function(b){return b.set(V,it)},this.setSize=function(b,k,B=!0){if(q.isPresenting){console.warn("THREE.WebGLRenderer: Can't change size while VR device is presenting.");return}V=b,it=k,e.width=Math.floor(b*W),e.height=Math.floor(k*W),B===!0&&(e.style.width=b+"px",e.style.height=k+"px"),this.setViewport(0,0,b,k)},this.getDrawingBufferSize=function(b){return b.set(V*W,it*W).floor()},this.setDrawingBufferSize=function(b,k,B){V=b,it=k,W=B,e.width=Math.floor(b*B),e.height=Math.floor(k*B),this.setViewport(0,0,b,k)},this.getCurrentViewport=function(b){return b.copy(R)},this.getViewport=function(b){return b.copy(Mt)},this.setViewport=function(b,k,B,O){b.isVector4?Mt.set(b.x,b.y,b.z,b.w):Mt.set(b,k,B,O),lt.viewport(R.copy(Mt).multiplyScalar(W).round())},this.getScissor=function(b){return b.copy(Yt)},this.setScissor=function(b,k,B,O){b.isVector4?Yt.set(b.x,b.y,b.z,b.w):Yt.set(b,k,B,O),lt.scissor(G.copy(Yt).multiplyScalar(W).round())},this.getScissorTest=function(){return he},this.setScissorTest=function(b){lt.setScissorTest(he=b)},this.setOpaqueSort=function(b){ct=b},this.setTransparentSort=function(b){gt=b},this.getClearColor=function(b){return b.copy(Ht.getClearColor())},this.setClearColor=function(){Ht.setClearColor.apply(Ht,arguments)},this.getClearAlpha=function(){return Ht.getClearAlpha()},this.setClearAlpha=function(){Ht.setClearAlpha.apply(Ht,arguments)},this.clear=function(b=!0,k=!0,B=!0){let O=0;if(b){let L=!1;if(P!==null){let at=P.texture.format;L=at===ou||at===au||at===ru}if(L){let at=P.texture.type,pt=at===Ai||at===Us||at===da||at===Cr||at===nu||at===iu,At=Ht.getClearColor(),Ct=Ht.getClearAlpha(),Xt=At.r,Jt=At.g,Rt=At.b;pt?(g[0]=Xt,g[1]=Jt,g[2]=Rt,g[3]=Ct,A.clearBufferuiv(A.COLOR,0,g)):(x[0]=Xt,x[1]=Jt,x[2]=Rt,x[3]=Ct,A.clearBufferiv(A.COLOR,0,x))}else O|=A.COLOR_BUFFER_BIT}k&&(O|=A.DEPTH_BUFFER_BIT),B&&(O|=A.STENCIL_BUFFER_BIT,this.state.buffers.stencil.setMask(4294967295)),A.clear(O)},this.clearColor=function(){this.clear(!0,!1,!1)},this.clearDepth=function(){this.clear(!1,!0,!1)},this.clearStencil=function(){this.clear(!1,!1,!0)},this.dispose=function(){e.removeEventListener("webglcontextlost",K,!1),e.removeEventListener("webglcontextrestored",mt,!1),e.removeEventListener("webglcontextcreationerror",ft,!1),vt.dispose(),ne.dispose(),xt.dispose(),v.dispose(),U.dispose(),$.dispose(),ve.dispose(),F.dispose(),Et.dispose(),q.dispose(),q.removeEventListener("sessionstart",zu),q.removeEventListener("sessionend",Hu),Es.stop()};function K(b){b.preventDefault(),console.log("THREE.WebGLRenderer: Context Lost."),D=!0}function mt(){console.log("THREE.WebGLRenderer: Context Restored."),D=!1;let b=zt.autoReset,k=_t.enabled,B=_t.autoUpdate,O=_t.needsUpdate,L=_t.type;dt(),zt.autoReset=b,_t.enabled=k,_t.autoUpdate=B,_t.needsUpdate=O,_t.type=L}function ft(b){console.error("THREE.WebGLRenderer: A WebGL context could not be created. Reason: ",b.statusMessage)}function Zt(b){let k=b.target;k.removeEventListener("dispose",Zt),Ie(k)}function Ie(b){Ye(b),xt.remove(b)}function Ye(b){let k=xt.get(b).programs;k!==void 0&&(k.forEach(function(B){Et.releaseProgram(B)}),b.isShaderMaterial&&Et.releaseShaderCache(b))}this.renderBufferDirect=function(b,k,B,O,L,at){k===null&&(k=le);let pt=L.isMesh&&L.matrixWorld.determinant()<0,At=wp(b,k,B,O,L);lt.setMaterial(O,pt);let Ct=B.index,Xt=1;if(O.wireframe===!0){if(Ct=tt.getWireframeAttribute(B),Ct===void 0)return;Xt=2}let Jt=B.drawRange,Rt=B.attributes.position,ce=Jt.start*Xt,ye=(Jt.start+Jt.count)*Xt;at!==null&&(ce=Math.max(ce,at.start*Xt),ye=Math.min(ye,(at.start+at.count)*Xt)),Ct!==null?(ce=Math.max(ce,0),ye=Math.min(ye,Ct.count)):Rt!=null&&(ce=Math.max(ce,0),ye=Math.min(ye,Rt.count));let be=ye-ce;if(be<0||be===1/0)return;ve.setup(L,O,At,B,Ct);let fn,ue=bt;if(Ct!==null&&(fn=X.get(Ct),ue=ae,ue.setIndex(fn)),L.isMesh)O.wireframe===!0?(lt.setLineWidth(O.wireframeLinewidth*st()),ue.setMode(A.LINES)):ue.setMode(A.TRIANGLES);else if(L.isLine){let Dt=O.linewidth;Dt===void 0&&(Dt=1),lt.setLineWidth(Dt*st()),L.isLineSegments?ue.setMode(A.LINES):L.isLineLoop?ue.setMode(A.LINE_LOOP):ue.setMode(A.LINE_STRIP)}else L.isPoints?ue.setMode(A.POINTS):L.isSprite&&ue.setMode(A.TRIANGLES);if(L.isBatchedMesh)if(L._multiDrawInstances!==null)ue.renderMultiDrawInstances(L._multiDrawStarts,L._multiDrawCounts,L._multiDrawCount,L._multiDrawInstances);else if(et.get("WEBGL_multi_draw"))ue.renderMultiDraw(L._multiDrawStarts,L._multiDrawCounts,L._multiDrawCount);else{let Dt=L._multiDrawStarts,mi=L._multiDrawCounts,de=L._multiDrawCount,Nn=Ct?X.get(Ct).bytesPerElement:1,js=xt.get(O).currentProgram.getUniforms();for(let mn=0;mn<de;mn++)js.setValue(A,"_gl_DrawID",mn),ue.render(Dt[mn]/Nn,mi[mn])}else if(L.isInstancedMesh)ue.renderInstances(ce,be,L.count);else if(B.isInstancedBufferGeometry){let Dt=B._maxInstanceCount!==void 0?B._maxInstanceCount:1/0,mi=Math.min(B.instanceCount,Dt);ue.renderInstances(ce,be,mi)}else ue.render(ce,be)};function me(b,k,B){b.transparent===!0&&b.side===vn&&b.forceSinglePass===!1?(b.side=je,b.needsUpdate=!0,Za(b,k,B),b.side=Qi,b.needsUpdate=!0,Za(b,k,B),b.side=vn):Za(b,k,B)}this.compile=function(b,k,B=null){B===null&&(B=b),p=ne.get(B),p.init(k),M.push(p),B.traverseVisible(function(L){L.isLight&&L.layers.test(k.layers)&&(p.pushLight(L),L.castShadow&&p.pushShadow(L))}),b!==B&&b.traverseVisible(function(L){L.isLight&&L.layers.test(k.layers)&&(p.pushLight(L),L.castShadow&&p.pushShadow(L))}),p.setupLights();let O=new Set;return b.traverse(function(L){if(!(L.isMesh||L.isPoints||L.isLine||L.isSprite))return;let at=L.material;if(at)if(Array.isArray(at))for(let pt=0;pt<at.length;pt++){let At=at[pt];me(At,B,L),O.add(At)}else me(at,B,L),O.add(at)}),M.pop(),p=null,O},this.compileAsync=function(b,k,B=null){let O=this.compile(b,k,B);return new Promise(L=>{function at(){if(O.forEach(function(pt){xt.get(pt).currentProgram.isReady()&&O.delete(pt)}),O.size===0){L(b);return}setTimeout(at,10)}et.get("KHR_parallel_shader_compile")!==null?at():setTimeout(at,10)})};let Un=null;function pi(b){Un&&Un(b)}function zu(){Es.stop()}function Hu(){Es.start()}let Es=new Sf;Es.setAnimationLoop(pi),typeof self<"u"&&Es.setContext(self),this.setAnimationLoop=function(b){Un=b,q.setAnimationLoop(b),b===null?Es.stop():Es.start()},q.addEventListener("sessionstart",zu),q.addEventListener("sessionend",Hu),this.render=function(b,k){if(k!==void 0&&k.isCamera!==!0){console.error("THREE.WebGLRenderer.render: camera is not an instance of THREE.Camera.");return}if(D===!0)return;if(b.matrixWorldAutoUpdate===!0&&b.updateMatrixWorld(),k.parent===null&&k.matrixWorldAutoUpdate===!0&&k.updateMatrixWorld(),q.enabled===!0&&q.isPresenting===!0&&(q.cameraAutoUpdate===!0&&q.updateCamera(k),k=q.getCamera()),b.isScene===!0&&b.onBeforeRender(y,b,k,P),p=ne.get(b,M.length),p.init(k),M.push(p),Bt.multiplyMatrices(k.projectionMatrix,k.matrixWorldInverse),Y.setFromProjectionMatrix(Bt),St=this.localClippingEnabled,rt=nt.init(this.clippingPlanes,St),m=vt.get(b,T.length),m.init(),T.push(m),q.enabled===!0&&q.isPresenting===!0){let at=y.xr.getDepthSensingMesh();at!==null&&Bl(at,k,-1/0,y.sortObjects)}Bl(b,k,0,y.sortObjects),m.finish(),y.sortObjects===!0&&m.sort(ct,gt),Q=q.enabled===!1||q.isPresenting===!1||q.hasDepthSensing()===!1,Q&&Ht.addToRenderList(m,b),this.info.render.frame++,rt===!0&&nt.beginShadows();let B=p.state.shadowsArray;_t.render(B,b,k),rt===!0&&nt.endShadows(),this.info.autoReset===!0&&this.info.reset();let O=m.opaque,L=m.transmissive;if(p.setupLights(),k.isArrayCamera){let at=k.cameras;if(L.length>0)for(let pt=0,At=at.length;pt<At;pt++){let Ct=at[pt];Gu(O,L,b,Ct)}Q&&Ht.render(b);for(let pt=0,At=at.length;pt<At;pt++){let Ct=at[pt];Vu(m,b,Ct,Ct.viewport)}}else L.length>0&&Gu(O,L,b,k),Q&&Ht.render(b),Vu(m,b,k);P!==null&&(E.updateMultisampleRenderTarget(P),E.updateRenderTargetMipmap(P)),b.isScene===!0&&b.onAfterRender(y,b,k),ve.resetDefaultState(),w=-1,_=null,M.pop(),M.length>0?(p=M[M.length-1],rt===!0&&nt.setGlobalState(y.clippingPlanes,p.state.camera)):p=null,T.pop(),T.length>0?m=T[T.length-1]:m=null};function Bl(b,k,B,O){if(b.visible===!1)return;if(b.layers.test(k.layers)){if(b.isGroup)B=b.renderOrder;else if(b.isLOD)b.autoUpdate===!0&&b.update(k);else if(b.isLight)p.pushLight(b),b.castShadow&&p.pushShadow(b);else if(b.isSprite){if(!b.frustumCulled||Y.intersectsSprite(b)){O&&Vt.setFromMatrixPosition(b.matrixWorld).applyMatrix4(Bt);let pt=$.update(b),At=b.material;At.visible&&m.push(b,pt,At,B,Vt.z,null)}}else if((b.isMesh||b.isLine||b.isPoints)&&(!b.frustumCulled||Y.intersectsObject(b))){let pt=$.update(b),At=b.material;if(O&&(b.boundingSphere!==void 0?(b.boundingSphere===null&&b.computeBoundingSphere(),Vt.copy(b.boundingSphere.center)):(pt.boundingSphere===null&&pt.computeBoundingSphere(),Vt.copy(pt.boundingSphere.center)),Vt.applyMatrix4(b.matrixWorld).applyMatrix4(Bt)),Array.isArray(At)){let Ct=pt.groups;for(let Xt=0,Jt=Ct.length;Xt<Jt;Xt++){let Rt=Ct[Xt],ce=At[Rt.materialIndex];ce&&ce.visible&&m.push(b,pt,ce,B,Vt.z,Rt)}}else At.visible&&m.push(b,pt,At,B,Vt.z,null)}}let at=b.children;for(let pt=0,At=at.length;pt<At;pt++)Bl(at[pt],k,B,O)}function Vu(b,k,B,O){let L=b.opaque,at=b.transmissive,pt=b.transparent;p.setupLightsView(B),rt===!0&&nt.setGlobalState(y.clippingPlanes,B),O&&lt.viewport(R.copy(O)),L.length>0&&Ya(L,k,B),at.length>0&&Ya(at,k,B),pt.length>0&&Ya(pt,k,B),lt.buffers.depth.setTest(!0),lt.buffers.depth.setMask(!0),lt.buffers.color.setMask(!0),lt.setPolygonOffset(!1)}function Gu(b,k,B,O){if((B.isScene===!0?B.overrideMaterial:null)!==null)return;p.state.transmissionRenderTarget[O.id]===void 0&&(p.state.transmissionRenderTarget[O.id]=new Ue(1,1,{generateMipmaps:!0,type:et.has("EXT_color_buffer_half_float")||et.has("EXT_color_buffer_float")?en:Ai,minFilter:Ls,samples:4,stencilBuffer:r,resolveDepthBuffer:!1,resolveStencilBuffer:!1,colorSpace:se.workingColorSpace}));let at=p.state.transmissionRenderTarget[O.id],pt=O.viewport||R;at.setSize(pt.z,pt.w);let At=y.getRenderTarget();y.setRenderTarget(at),y.getClearColor(z),J=y.getClearAlpha(),J<1&&y.setClearColor(16777215,.5),y.clear(),Q&&Ht.render(B);let Ct=y.toneMapping;y.toneMapping=Ji;let Xt=O.viewport;if(O.viewport!==void 0&&(O.viewport=void 0),p.setupLightsView(O),rt===!0&&nt.setGlobalState(y.clippingPlanes,O),Ya(b,B,O),E.updateMultisampleRenderTarget(at),E.updateRenderTargetMipmap(at),et.has("WEBGL_multisampled_render_to_texture")===!1){let Jt=!1;for(let Rt=0,ce=k.length;Rt<ce;Rt++){let ye=k[Rt],be=ye.object,fn=ye.geometry,ue=ye.material,Dt=ye.group;if(ue.side===vn&&be.layers.test(O.layers)){let mi=ue.side;ue.side=je,ue.needsUpdate=!0,Wu(be,B,O,fn,ue,Dt),ue.side=mi,ue.needsUpdate=!0,Jt=!0}}Jt===!0&&(E.updateMultisampleRenderTarget(at),E.updateRenderTargetMipmap(at))}y.setRenderTarget(At),y.setClearColor(z,J),Xt!==void 0&&(O.viewport=Xt),y.toneMapping=Ct}function Ya(b,k,B){let O=k.isScene===!0?k.overrideMaterial:null;for(let L=0,at=b.length;L<at;L++){let pt=b[L],At=pt.object,Ct=pt.geometry,Xt=O===null?pt.material:O,Jt=pt.group;At.layers.test(B.layers)&&Wu(At,k,B,Ct,Xt,Jt)}}function Wu(b,k,B,O,L,at){b.onBeforeRender(y,k,B,O,L,at),b.modelViewMatrix.multiplyMatrices(B.matrixWorldInverse,b.matrixWorld),b.normalMatrix.getNormalMatrix(b.modelViewMatrix),L.onBeforeRender(y,k,B,O,b,at),L.transparent===!0&&L.side===vn&&L.forceSinglePass===!1?(L.side=je,L.needsUpdate=!0,y.renderBufferDirect(B,k,O,L,b,at),L.side=Qi,L.needsUpdate=!0,y.renderBufferDirect(B,k,O,L,b,at),L.side=vn):y.renderBufferDirect(B,k,O,L,b,at),b.onAfterRender(y,k,B,O,L,at)}function Za(b,k,B){k.isScene!==!0&&(k=le);let O=xt.get(b),L=p.state.lights,at=p.state.shadowsArray,pt=L.state.version,At=Et.getParameters(b,L.state,at,k,B),Ct=Et.getProgramCacheKey(At),Xt=O.programs;O.environment=b.isMeshStandardMaterial?k.environment:null,O.fog=k.fog,O.envMap=(b.isMeshStandardMaterial?U:v).get(b.envMap||O.environment),O.envMapRotation=O.environment!==null&&b.envMap===null?k.environmentRotation:b.envMapRotation,Xt===void 0&&(b.addEventListener("dispose",Zt),Xt=new Map,O.programs=Xt);let Jt=Xt.get(Ct);if(Jt!==void 0){if(O.currentProgram===Jt&&O.lightsStateVersion===pt)return Xu(b,At),Jt}else At.uniforms=Et.getUniforms(b),b.onBeforeCompile(At,y),Jt=Et.acquireProgram(At,Ct),Xt.set(Ct,Jt),O.uniforms=At.uniforms;let Rt=O.uniforms;return(!b.isShaderMaterial&&!b.isRawShaderMaterial||b.clipping===!0)&&(Rt.clippingPlanes=nt.uniform),Xu(b,At),O.needsLights=Tp(b),O.lightsStateVersion=pt,O.needsLights&&(Rt.ambientLightColor.value=L.state.ambient,Rt.lightProbe.value=L.state.probe,Rt.directionalLights.value=L.state.directional,Rt.directionalLightShadows.value=L.state.directionalShadow,Rt.spotLights.value=L.state.spot,Rt.spotLightShadows.value=L.state.spotShadow,Rt.rectAreaLights.value=L.state.rectArea,Rt.ltc_1.value=L.state.rectAreaLTC1,Rt.ltc_2.value=L.state.rectAreaLTC2,Rt.pointLights.value=L.state.point,Rt.pointLightShadows.value=L.state.pointShadow,Rt.hemisphereLights.value=L.state.hemi,Rt.directionalShadowMap.value=L.state.directionalShadowMap,Rt.directionalShadowMatrix.value=L.state.directionalShadowMatrix,Rt.spotShadowMap.value=L.state.spotShadowMap,Rt.spotLightMatrix.value=L.state.spotLightMatrix,Rt.spotLightMap.value=L.state.spotLightMap,Rt.pointShadowMap.value=L.state.pointShadowMap,Rt.pointShadowMatrix.value=L.state.pointShadowMatrix),O.currentProgram=Jt,O.uniformsList=null,Jt}function qu(b){if(b.uniformsList===null){let k=b.currentProgram.getUniforms();b.uniformsList=wr.seqWithValue(k.seq,b.uniforms)}return b.uniformsList}function Xu(b,k){let B=xt.get(b);B.outputColorSpace=k.outputColorSpace,B.batching=k.batching,B.batchingColor=k.batchingColor,B.instancing=k.instancing,B.instancingColor=k.instancingColor,B.instancingMorph=k.instancingMorph,B.skinning=k.skinning,B.morphTargets=k.morphTargets,B.morphNormals=k.morphNormals,B.morphColors=k.morphColors,B.morphTargetsCount=k.morphTargetsCount,B.numClippingPlanes=k.numClippingPlanes,B.numIntersection=k.numClipIntersection,B.vertexAlphas=k.vertexAlphas,B.vertexTangents=k.vertexTangents,B.toneMapping=k.toneMapping}function wp(b,k,B,O,L){k.isScene!==!0&&(k=le),E.resetTextureUnits();let at=k.fog,pt=O.isMeshStandardMaterial?k.environment:null,At=P===null?y.outputColorSpace:P.isXRRenderTarget===!0?P.texture.colorSpace:Lr,Ct=(O.isMeshStandardMaterial?U:v).get(O.envMap||pt),Xt=O.vertexColors===!0&&!!B.attributes.color&&B.attributes.color.itemSize===4,Jt=!!B.attributes.tangent&&(!!O.normalMap||O.anisotropy>0),Rt=!!B.morphAttributes.position,ce=!!B.morphAttributes.normal,ye=!!B.morphAttributes.color,be=Ji;O.toneMapped&&(P===null||P.isXRRenderTarget===!0)&&(be=y.toneMapping);let fn=B.morphAttributes.position||B.morphAttributes.normal||B.morphAttributes.color,ue=fn!==void 0?fn.length:0,Dt=xt.get(O),mi=p.state.lights;if(rt===!0&&(St===!0||b!==_)){let An=b===_&&O.id===w;nt.setState(O,b,An)}let de=!1;O.version===Dt.__version?(Dt.needsLights&&Dt.lightsStateVersion!==mi.state.version||Dt.outputColorSpace!==At||L.isBatchedMesh&&Dt.batching===!1||!L.isBatchedMesh&&Dt.batching===!0||L.isBatchedMesh&&Dt.batchingColor===!0&&L.colorTexture===null||L.isBatchedMesh&&Dt.batchingColor===!1&&L.colorTexture!==null||L.isInstancedMesh&&Dt.instancing===!1||!L.isInstancedMesh&&Dt.instancing===!0||L.isSkinnedMesh&&Dt.skinning===!1||!L.isSkinnedMesh&&Dt.skinning===!0||L.isInstancedMesh&&Dt.instancingColor===!0&&L.instanceColor===null||L.isInstancedMesh&&Dt.instancingColor===!1&&L.instanceColor!==null||L.isInstancedMesh&&Dt.instancingMorph===!0&&L.morphTexture===null||L.isInstancedMesh&&Dt.instancingMorph===!1&&L.morphTexture!==null||Dt.envMap!==Ct||O.fog===!0&&Dt.fog!==at||Dt.numClippingPlanes!==void 0&&(Dt.numClippingPlanes!==nt.numPlanes||Dt.numIntersection!==nt.numIntersection)||Dt.vertexAlphas!==Xt||Dt.vertexTangents!==Jt||Dt.morphTargets!==Rt||Dt.morphNormals!==ce||Dt.morphColors!==ye||Dt.toneMapping!==be||Dt.morphTargetsCount!==ue)&&(de=!0):(de=!0,Dt.__version=O.version);let Nn=Dt.currentProgram;de===!0&&(Nn=Za(O,k,L));let js=!1,mn=!1,Yr=!1,Me=Nn.getUniforms(),$n=Dt.uniforms;if(lt.useProgram(Nn.program)&&(js=!0,mn=!0,Yr=!0),O.id!==w&&(w=O.id,mn=!0),js||_!==b){lt.buffers.depth.getReversed()?(ot.copy(b.projectionMatrix),lm(ot),cm(ot),Me.setValue(A,"projectionMatrix",ot)):Me.setValue(A,"projectionMatrix",b.projectionMatrix),Me.setValue(A,"viewMatrix",b.matrixWorldInverse);let zi=Me.map.cameraPosition;zi!==void 0&&zi.setValue(A,qt.setFromMatrixPosition(b.matrixWorld)),yt.logarithmicDepthBuffer&&Me.setValue(A,"logDepthBufFC",2/(Math.log(b.far+1)/Math.LN2)),(O.isMeshPhongMaterial||O.isMeshToonMaterial||O.isMeshLambertMaterial||O.isMeshBasicMaterial||O.isMeshStandardMaterial||O.isShaderMaterial)&&Me.setValue(A,"isOrthographic",b.isOrthographicCamera===!0),_!==b&&(_=b,mn=!0,Yr=!0)}if(L.isSkinnedMesh){Me.setOptional(A,L,"bindMatrix"),Me.setOptional(A,L,"bindMatrixInverse");let An=L.skeleton;An&&(An.boneTexture===null&&An.computeBoneTexture(),Me.setValue(A,"boneTexture",An.boneTexture,E))}L.isBatchedMesh&&(Me.setOptional(A,L,"batchingTexture"),Me.setValue(A,"batchingTexture",L._matricesTexture,E),Me.setOptional(A,L,"batchingIdTexture"),Me.setValue(A,"batchingIdTexture",L._indirectTexture,E),Me.setOptional(A,L,"batchingColorTexture"),L._colorsTexture!==null&&Me.setValue(A,"batchingColorTexture",L._colorsTexture,E));let Zr=B.morphAttributes;if((Zr.position!==void 0||Zr.normal!==void 0||Zr.color!==void 0)&&Wt.update(L,B,Nn),(mn||Dt.receiveShadow!==L.receiveShadow)&&(Dt.receiveShadow=L.receiveShadow,Me.setValue(A,"receiveShadow",L.receiveShadow)),O.isMeshGouraudMaterial&&O.envMap!==null&&($n.envMap.value=Ct,$n.flipEnvMap.value=Ct.isCubeTexture&&Ct.isRenderTargetTexture===!1?-1:1),O.isMeshStandardMaterial&&O.envMap===null&&k.environment!==null&&($n.envMapIntensity.value=k.environmentIntensity),mn&&(Me.setValue(A,"toneMappingExposure",y.toneMappingExposure),Dt.needsLights&&Ep($n,Yr),at&&O.fog===!0&&ut.refreshFogUniforms($n,at),ut.refreshMaterialUniforms($n,O,W,it,p.state.transmissionRenderTarget[b.id]),wr.upload(A,qu(Dt),$n,E)),O.isShaderMaterial&&O.uniformsNeedUpdate===!0&&(wr.upload(A,qu(Dt),$n,E),O.uniformsNeedUpdate=!1),O.isSpriteMaterial&&Me.setValue(A,"center",L.center),Me.setValue(A,"modelViewMatrix",L.modelViewMatrix),Me.setValue(A,"normalMatrix",L.normalMatrix),Me.setValue(A,"modelMatrix",L.matrixWorld),O.isShaderMaterial||O.isRawShaderMaterial){let An=O.uniformsGroups;for(let zi=0,Hi=An.length;zi<Hi;zi++){let $u=An[zi];F.update($u,Nn),F.bind($u,Nn)}}return Nn}function Ep(b,k){b.ambientLightColor.needsUpdate=k,b.lightProbe.needsUpdate=k,b.directionalLights.needsUpdate=k,b.directionalLightShadows.needsUpdate=k,b.pointLights.needsUpdate=k,b.pointLightShadows.needsUpdate=k,b.spotLights.needsUpdate=k,b.spotLightShadows.needsUpdate=k,b.rectAreaLights.needsUpdate=k,b.hemisphereLights.needsUpdate=k}function Tp(b){return b.isMeshLambertMaterial||b.isMeshToonMaterial||b.isMeshPhongMaterial||b.isMeshStandardMaterial||b.isShadowMaterial||b.isShaderMaterial&&b.lights===!0}this.getActiveCubeFace=function(){return S},this.getActiveMipmapLevel=function(){return C},this.getRenderTarget=function(){return P},this.setRenderTargetTextures=function(b,k,B){xt.get(b.texture).__webglTexture=k,xt.get(b.depthTexture).__webglTexture=B;let O=xt.get(b);O.__hasExternalTextures=!0,O.__autoAllocateDepthBuffer=B===void 0,O.__autoAllocateDepthBuffer||et.has("WEBGL_multisampled_render_to_texture")===!0&&(console.warn("THREE.WebGLRenderer: Render-to-texture extension was disabled because an external texture was provided"),O.__useRenderToTexture=!1)},this.setRenderTargetFramebuffer=function(b,k){let B=xt.get(b);B.__webglFramebuffer=k,B.__useDefaultFramebuffer=k===void 0},this.setRenderTarget=function(b,k=0,B=0){P=b,S=k,C=B;let O=!0,L=null,at=!1,pt=!1;if(b){let Ct=xt.get(b);if(Ct.__useDefaultFramebuffer!==void 0)lt.bindFramebuffer(A.FRAMEBUFFER,null),O=!1;else if(Ct.__webglFramebuffer===void 0)E.setupRenderTarget(b);else if(Ct.__hasExternalTextures)E.rebindTextures(b,xt.get(b.texture).__webglTexture,xt.get(b.depthTexture).__webglTexture);else if(b.depthBuffer){let Rt=b.depthTexture;if(Ct.__boundDepthTexture!==Rt){if(Rt!==null&&xt.has(Rt)&&(b.width!==Rt.image.width||b.height!==Rt.image.height))throw new Error("WebGLRenderTarget: Attached DepthTexture is initialized to the incorrect size.");E.setupDepthRenderbuffer(b)}}let Xt=b.texture;(Xt.isData3DTexture||Xt.isDataArrayTexture||Xt.isCompressedArrayTexture)&&(pt=!0);let Jt=xt.get(b).__webglFramebuffer;b.isWebGLCubeRenderTarget?(Array.isArray(Jt[k])?L=Jt[k][B]:L=Jt[k],at=!0):b.samples>0&&E.useMultisampledRTT(b)===!1?L=xt.get(b).__webglMultisampledFramebuffer:Array.isArray(Jt)?L=Jt[B]:L=Jt,R.copy(b.viewport),G.copy(b.scissor),N=b.scissorTest}else R.copy(Mt).multiplyScalar(W).floor(),G.copy(Yt).multiplyScalar(W).floor(),N=he;if(lt.bindFramebuffer(A.FRAMEBUFFER,L)&&O&&lt.drawBuffers(b,L),lt.viewport(R),lt.scissor(G),lt.setScissorTest(N),at){let Ct=xt.get(b.texture);A.framebufferTexture2D(A.FRAMEBUFFER,A.COLOR_ATTACHMENT0,A.TEXTURE_CUBE_MAP_POSITIVE_X+k,Ct.__webglTexture,B)}else if(pt){let Ct=xt.get(b.texture),Xt=k||0;A.framebufferTextureLayer(A.FRAMEBUFFER,A.COLOR_ATTACHMENT0,Ct.__webglTexture,B||0,Xt)}w=-1},this.readRenderTargetPixels=function(b,k,B,O,L,at,pt){if(!(b&&b.isWebGLRenderTarget)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");return}let At=xt.get(b).__webglFramebuffer;if(b.isWebGLCubeRenderTarget&&pt!==void 0&&(At=At[pt]),At){lt.bindFramebuffer(A.FRAMEBUFFER,At);try{let Ct=b.texture,Xt=Ct.format,Jt=Ct.type;if(!yt.textureFormatReadable(Xt)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not in RGBA or implementation defined format.");return}if(!yt.textureTypeReadable(Jt)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not in UnsignedByteType or implementation defined type.");return}k>=0&&k<=b.width-O&&B>=0&&B<=b.height-L&&A.readPixels(k,B,O,L,jt.convert(Xt),jt.convert(Jt),at)}finally{let Ct=P!==null?xt.get(P).__webglFramebuffer:null;lt.bindFramebuffer(A.FRAMEBUFFER,Ct)}}},this.readRenderTargetPixelsAsync=async function(b,k,B,O,L,at,pt){if(!(b&&b.isWebGLRenderTarget))throw new Error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");let At=xt.get(b).__webglFramebuffer;if(b.isWebGLCubeRenderTarget&&pt!==void 0&&(At=At[pt]),At){let Ct=b.texture,Xt=Ct.format,Jt=Ct.type;if(!yt.textureFormatReadable(Xt))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in RGBA or implementation defined format.");if(!yt.textureTypeReadable(Jt))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in UnsignedByteType or implementation defined type.");if(k>=0&&k<=b.width-O&&B>=0&&B<=b.height-L){lt.bindFramebuffer(A.FRAMEBUFFER,At);let Rt=A.createBuffer();A.bindBuffer(A.PIXEL_PACK_BUFFER,Rt),A.bufferData(A.PIXEL_PACK_BUFFER,at.byteLength,A.STREAM_READ),A.readPixels(k,B,O,L,jt.convert(Xt),jt.convert(Jt),0);let ce=P!==null?xt.get(P).__webglFramebuffer:null;lt.bindFramebuffer(A.FRAMEBUFFER,ce);let ye=A.fenceSync(A.SYNC_GPU_COMMANDS_COMPLETE,0);return A.flush(),await om(A,ye,4),A.bindBuffer(A.PIXEL_PACK_BUFFER,Rt),A.getBufferSubData(A.PIXEL_PACK_BUFFER,0,at),A.deleteBuffer(Rt),A.deleteSync(ye),at}else throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: requested read bounds are out of range.")}},this.copyFramebufferToTexture=function(b,k=null,B=0){b.isTexture!==!0&&(aa("WebGLRenderer: copyFramebufferToTexture function signature has changed."),k=arguments[0]||null,b=arguments[1]);let O=Math.pow(2,-B),L=Math.floor(b.image.width*O),at=Math.floor(b.image.height*O),pt=k!==null?k.x:0,At=k!==null?k.y:0;E.setTexture2D(b,0),A.copyTexSubImage2D(A.TEXTURE_2D,B,0,0,pt,At,L,at),lt.unbindTexture()},this.copyTextureToTexture=function(b,k,B=null,O=null,L=0){b.isTexture!==!0&&(aa("WebGLRenderer: copyTextureToTexture function signature has changed."),O=arguments[0]||null,b=arguments[1],k=arguments[2],L=arguments[3]||0,B=null);let at,pt,At,Ct,Xt,Jt,Rt,ce,ye,be=b.isCompressedTexture?b.mipmaps[L]:b.image;B!==null?(at=B.max.x-B.min.x,pt=B.max.y-B.min.y,At=B.isBox3?B.max.z-B.min.z:1,Ct=B.min.x,Xt=B.min.y,Jt=B.isBox3?B.min.z:0):(at=be.width,pt=be.height,At=be.depth||1,Ct=0,Xt=0,Jt=0),O!==null?(Rt=O.x,ce=O.y,ye=O.z):(Rt=0,ce=0,ye=0);let fn=jt.convert(k.format),ue=jt.convert(k.type),Dt;k.isData3DTexture?(E.setTexture3D(k,0),Dt=A.TEXTURE_3D):k.isDataArrayTexture||k.isCompressedArrayTexture?(E.setTexture2DArray(k,0),Dt=A.TEXTURE_2D_ARRAY):(E.setTexture2D(k,0),Dt=A.TEXTURE_2D),A.pixelStorei(A.UNPACK_FLIP_Y_WEBGL,k.flipY),A.pixelStorei(A.UNPACK_PREMULTIPLY_ALPHA_WEBGL,k.premultiplyAlpha),A.pixelStorei(A.UNPACK_ALIGNMENT,k.unpackAlignment);let mi=A.getParameter(A.UNPACK_ROW_LENGTH),de=A.getParameter(A.UNPACK_IMAGE_HEIGHT),Nn=A.getParameter(A.UNPACK_SKIP_PIXELS),js=A.getParameter(A.UNPACK_SKIP_ROWS),mn=A.getParameter(A.UNPACK_SKIP_IMAGES);A.pixelStorei(A.UNPACK_ROW_LENGTH,be.width),A.pixelStorei(A.UNPACK_IMAGE_HEIGHT,be.height),A.pixelStorei(A.UNPACK_SKIP_PIXELS,Ct),A.pixelStorei(A.UNPACK_SKIP_ROWS,Xt),A.pixelStorei(A.UNPACK_SKIP_IMAGES,Jt);let Yr=b.isDataArrayTexture||b.isData3DTexture,Me=k.isDataArrayTexture||k.isData3DTexture;if(b.isRenderTargetTexture||b.isDepthTexture){let $n=xt.get(b),Zr=xt.get(k),An=xt.get($n.__renderTarget),zi=xt.get(Zr.__renderTarget);lt.bindFramebuffer(A.READ_FRAMEBUFFER,An.__webglFramebuffer),lt.bindFramebuffer(A.DRAW_FRAMEBUFFER,zi.__webglFramebuffer);for(let Hi=0;Hi<At;Hi++)Yr&&A.framebufferTextureLayer(A.READ_FRAMEBUFFER,A.COLOR_ATTACHMENT0,xt.get(b).__webglTexture,L,Jt+Hi),b.isDepthTexture?(Me&&A.framebufferTextureLayer(A.DRAW_FRAMEBUFFER,A.COLOR_ATTACHMENT0,xt.get(k).__webglTexture,L,ye+Hi),A.blitFramebuffer(Ct,Xt,at,pt,Rt,ce,at,pt,A.DEPTH_BUFFER_BIT,A.NEAREST)):Me?A.copyTexSubImage3D(Dt,L,Rt,ce,ye+Hi,Ct,Xt,at,pt):A.copyTexSubImage2D(Dt,L,Rt,ce,ye+Hi,Ct,Xt,at,pt);lt.bindFramebuffer(A.READ_FRAMEBUFFER,null),lt.bindFramebuffer(A.DRAW_FRAMEBUFFER,null)}else Me?b.isDataTexture||b.isData3DTexture?A.texSubImage3D(Dt,L,Rt,ce,ye,at,pt,At,fn,ue,be.data):k.isCompressedArrayTexture?A.compressedTexSubImage3D(Dt,L,Rt,ce,ye,at,pt,At,fn,be.data):A.texSubImage3D(Dt,L,Rt,ce,ye,at,pt,At,fn,ue,be):b.isDataTexture?A.texSubImage2D(A.TEXTURE_2D,L,Rt,ce,at,pt,fn,ue,be.data):b.isCompressedTexture?A.compressedTexSubImage2D(A.TEXTURE_2D,L,Rt,ce,be.width,be.height,fn,be.data):A.texSubImage2D(A.TEXTURE_2D,L,Rt,ce,at,pt,fn,ue,be);A.pixelStorei(A.UNPACK_ROW_LENGTH,mi),A.pixelStorei(A.UNPACK_IMAGE_HEIGHT,de),A.pixelStorei(A.UNPACK_SKIP_PIXELS,Nn),A.pixelStorei(A.UNPACK_SKIP_ROWS,js),A.pixelStorei(A.UNPACK_SKIP_IMAGES,mn),L===0&&k.generateMipmaps&&A.generateMipmap(Dt),lt.unbindTexture()},this.copyTextureToTexture3D=function(b,k,B=null,O=null,L=0){return b.isTexture!==!0&&(aa("WebGLRenderer: copyTextureToTexture3D function signature has changed."),B=arguments[0]||null,O=arguments[1]||null,b=arguments[2],k=arguments[3],L=arguments[4]||0),aa('WebGLRenderer: copyTextureToTexture3D function has been deprecated. Use "copyTextureToTexture" instead.'),this.copyTextureToTexture(b,k,B,O,L)},this.initRenderTarget=function(b){xt.get(b).__webglFramebuffer===void 0&&E.setupRenderTarget(b)},this.initTexture=function(b){b.isCubeTexture?E.setTextureCube(b,0):b.isData3DTexture?E.setTexture3D(b,0):b.isDataArrayTexture||b.isCompressedArrayTexture?E.setTexture2DArray(b,0):E.setTexture2D(b,0),lt.unbindTexture()},this.resetState=function(){S=0,C=0,P=null,lt.reset(),ve.reset()},typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}get coordinateSystem(){return wi}get outputColorSpace(){return this._outputColorSpace}set outputColorSpace(t){this._outputColorSpace=t;let e=this.getContext();e.drawingBufferColorspace=se._getDrawingBufferColorSpace(t),e.unpackColorSpace=se._getUnpackColorSpace()}};var ma=class extends ln{constructor(){super(),this.isScene=!0,this.type="Scene",this.background=null,this.environment=null,this.fog=null,this.backgroundBlurriness=0,this.backgroundIntensity=1,this.backgroundRotation=new ti,this.environmentIntensity=1,this.environmentRotation=new ti,this.overrideMaterial=null,typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}copy(t,e){return super.copy(t,e),t.background!==null&&(this.background=t.background.clone()),t.environment!==null&&(this.environment=t.environment.clone()),t.fog!==null&&(this.fog=t.fog.clone()),this.backgroundBlurriness=t.backgroundBlurriness,this.backgroundIntensity=t.backgroundIntensity,this.backgroundRotation.copy(t.backgroundRotation),this.environmentIntensity=t.environmentIntensity,this.environmentRotation.copy(t.environmentRotation),t.overrideMaterial!==null&&(this.overrideMaterial=t.overrideMaterial.clone()),this.matrixAutoUpdate=t.matrixAutoUpdate,this}toJSON(t){let e=super.toJSON(t);return this.fog!==null&&(e.object.fog=this.fog.toJSON()),this.backgroundBlurriness>0&&(e.object.backgroundBlurriness=this.backgroundBlurriness),this.backgroundIntensity!==1&&(e.object.backgroundIntensity=this.backgroundIntensity),e.object.backgroundRotation=this.backgroundRotation.toArray(),this.environmentIntensity!==1&&(e.object.environmentIntensity=this.environmentIntensity),e.object.environmentRotation=this.environmentRotation.toArray(),e}},Eh=class{constructor(t,e){this.isInterleavedBuffer=!0,this.array=t,this.stride=e,this.count=t!==void 0?t.length/e:0,this.usage=ah,this.updateRanges=[],this.version=0,this.uuid=Ei()}onUploadCallback(){}set needsUpdate(t){t===!0&&this.version++}setUsage(t){return this.usage=t,this}addUpdateRange(t,e){this.updateRanges.push({start:t,count:e})}clearUpdateRanges(){this.updateRanges.length=0}copy(t){return this.array=new t.array.constructor(t.array),this.count=t.count,this.stride=t.stride,this.usage=t.usage,this}copyAt(t,e,i){t*=this.stride,i*=e.stride;for(let s=0,r=this.stride;s<r;s++)this.array[t+s]=e.array[i+s];return this}set(t,e=0){return this.array.set(t,e),this}clone(t){t.arrayBuffers===void 0&&(t.arrayBuffers={}),this.array.buffer._uuid===void 0&&(this.array.buffer._uuid=Ei()),t.arrayBuffers[this.array.buffer._uuid]===void 0&&(t.arrayBuffers[this.array.buffer._uuid]=this.array.slice(0).buffer);let e=new this.array.constructor(t.arrayBuffers[this.array.buffer._uuid]),i=new this.constructor(e,this.stride);return i.setUsage(this.usage),i}onUpload(t){return this.onUploadCallback=t,this}toJSON(t){return t.arrayBuffers===void 0&&(t.arrayBuffers={}),this.array.buffer._uuid===void 0&&(this.array.buffer._uuid=Ei()),t.arrayBuffers[this.array.buffer._uuid]===void 0&&(t.arrayBuffers[this.array.buffer._uuid]=Array.from(new Uint32Array(this.array.buffer))),{uuid:this.uuid,buffer:this.array.buffer._uuid,type:this.array.constructor.name,stride:this.stride}}},sn=new I,Vo=class n{constructor(t,e,i,s=!1){this.isInterleavedBufferAttribute=!0,this.name="",this.data=t,this.itemSize=e,this.offset=i,this.normalized=s}get count(){return this.data.count}get array(){return this.data.array}set needsUpdate(t){this.data.needsUpdate=t}applyMatrix4(t){for(let e=0,i=this.data.count;e<i;e++)sn.fromBufferAttribute(this,e),sn.applyMatrix4(t),this.setXYZ(e,sn.x,sn.y,sn.z);return this}applyNormalMatrix(t){for(let e=0,i=this.count;e<i;e++)sn.fromBufferAttribute(this,e),sn.applyNormalMatrix(t),this.setXYZ(e,sn.x,sn.y,sn.z);return this}transformDirection(t){for(let e=0,i=this.count;e<i;e++)sn.fromBufferAttribute(this,e),sn.transformDirection(t),this.setXYZ(e,sn.x,sn.y,sn.z);return this}getComponent(t,e){let i=this.array[t*this.data.stride+this.offset+e];return this.normalized&&(i=Zn(i,this.array)),i}setComponent(t,e,i){return this.normalized&&(i=ge(i,this.array)),this.data.array[t*this.data.stride+this.offset+e]=i,this}setX(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset]=e,this}setY(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset+1]=e,this}setZ(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset+2]=e,this}setW(t,e){return this.normalized&&(e=ge(e,this.array)),this.data.array[t*this.data.stride+this.offset+3]=e,this}getX(t){let e=this.data.array[t*this.data.stride+this.offset];return this.normalized&&(e=Zn(e,this.array)),e}getY(t){let e=this.data.array[t*this.data.stride+this.offset+1];return this.normalized&&(e=Zn(e,this.array)),e}getZ(t){let e=this.data.array[t*this.data.stride+this.offset+2];return this.normalized&&(e=Zn(e,this.array)),e}getW(t){let e=this.data.array[t*this.data.stride+this.offset+3];return this.normalized&&(e=Zn(e,this.array)),e}setXY(t,e,i){return t=t*this.data.stride+this.offset,this.normalized&&(e=ge(e,this.array),i=ge(i,this.array)),this.data.array[t+0]=e,this.data.array[t+1]=i,this}setXYZ(t,e,i,s){return t=t*this.data.stride+this.offset,this.normalized&&(e=ge(e,this.array),i=ge(i,this.array),s=ge(s,this.array)),this.data.array[t+0]=e,this.data.array[t+1]=i,this.data.array[t+2]=s,this}setXYZW(t,e,i,s,r){return t=t*this.data.stride+this.offset,this.normalized&&(e=ge(e,this.array),i=ge(i,this.array),s=ge(s,this.array),r=ge(r,this.array)),this.data.array[t+0]=e,this.data.array[t+1]=i,this.data.array[t+2]=s,this.data.array[t+3]=r,this}clone(t){if(t===void 0){console.log("THREE.InterleavedBufferAttribute.clone(): Cloning an interleaved buffer attribute will de-interleave buffer data.");let e=[];for(let i=0;i<this.count;i++){let s=i*this.data.stride+this.offset;for(let r=0;r<this.itemSize;r++)e.push(this.data.array[s+r])}return new an(new this.array.constructor(e),this.itemSize,this.normalized)}else return t.interleavedBuffers===void 0&&(t.interleavedBuffers={}),t.interleavedBuffers[this.data.uuid]===void 0&&(t.interleavedBuffers[this.data.uuid]=this.data.clone(t)),new n(t.interleavedBuffers[this.data.uuid],this.itemSize,this.offset,this.normalized)}toJSON(t){if(t===void 0){console.log("THREE.InterleavedBufferAttribute.toJSON(): Serializing an interleaved buffer attribute will de-interleave buffer data.");let e=[];for(let i=0;i<this.count;i++){let s=i*this.data.stride+this.offset;for(let r=0;r<this.itemSize;r++)e.push(this.data.array[s+r])}return{itemSize:this.itemSize,type:this.array.constructor.name,array:e,normalized:this.normalized}}else return t.interleavedBuffers===void 0&&(t.interleavedBuffers={}),t.interleavedBuffers[this.data.uuid]===void 0&&(t.interleavedBuffers[this.data.uuid]=this.data.toJSON(t)),{isInterleavedBufferAttribute:!0,itemSize:this.itemSize,data:this.data.uuid,offset:this.offset,normalized:this.normalized}}},ga=class extends es{static get type(){return"SpriteMaterial"}constructor(t){super(),this.isSpriteMaterial=!0,this.color=new Pt(16777215),this.map=null,this.alphaMap=null,this.rotation=0,this.sizeAttenuation=!0,this.transparent=!0,this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.alphaMap=t.alphaMap,this.rotation=t.rotation,this.sizeAttenuation=t.sizeAttenuation,this.fog=t.fog,this}},pr,ta=new I,mr=new I,gr=new I,xr=new j,ea=new j,Cf=new pe,vo=new I,na=new I,yo=new I,Xd=new j,mc=new j,$d=new j,Go=class extends ln{constructor(t=new ga){if(super(),this.isSprite=!0,this.type="Sprite",pr===void 0){pr=new tn;let e=new Float32Array([-.5,-.5,0,0,0,.5,-.5,0,1,0,.5,.5,0,1,1,-.5,.5,0,0,1]),i=new Eh(e,5);pr.setIndex([0,1,2,0,2,3]),pr.setAttribute("position",new Vo(i,3,0,!1)),pr.setAttribute("uv",new Vo(i,2,3,!1))}this.geometry=pr,this.material=t,this.center=new j(.5,.5)}raycast(t,e){t.camera===null&&console.error('THREE.Sprite: "Raycaster.camera" needs to be set in order to raycast against sprites.'),mr.setFromMatrixScale(this.matrixWorld),Cf.copy(t.camera.matrixWorld),this.modelViewMatrix.multiplyMatrices(t.camera.matrixWorldInverse,this.matrixWorld),gr.setFromMatrixPosition(this.modelViewMatrix),t.camera.isPerspectiveCamera&&this.material.sizeAttenuation===!1&&mr.multiplyScalar(-gr.z);let i=this.material.rotation,s,r;i!==0&&(r=Math.cos(i),s=Math.sin(i));let a=this.center;_o(vo.set(-.5,-.5,0),gr,a,mr,s,r),_o(na.set(.5,-.5,0),gr,a,mr,s,r),_o(yo.set(.5,.5,0),gr,a,mr,s,r),Xd.set(0,0),mc.set(1,0),$d.set(1,1);let o=t.ray.intersectTriangle(vo,na,yo,!1,ta);if(o===null&&(_o(na.set(-.5,.5,0),gr,a,mr,s,r),mc.set(0,1),o=t.ray.intersectTriangle(vo,yo,na,!1,ta),o===null))return;let l=t.ray.origin.distanceTo(ta);l<t.near||l>t.far||e.push({distance:l,point:ta.clone(),uv:Zi.getInterpolation(ta,vo,na,yo,Xd,mc,$d,new j),face:null,object:this})}copy(t,e){return super.copy(t,e),t.center!==void 0&&this.center.copy(t.center),this.material=t.material,this}};function _o(n,t,e,i,s,r){xr.subVectors(n,e).addScalar(.5).multiply(i),s!==void 0?(ea.x=r*xr.x-s*xr.y,ea.y=s*xr.x+r*xr.y):ea.copy(xr),n.copy(t),n.x+=ea.x,n.y+=ea.y,n.applyMatrix4(Cf)}var Th=class extends on{constructor(t=null,e=1,i=1,s,r,a,o,l,c=yn,h=yn,u,d){super(null,a,o,l,c,h,s,r,u,d),this.isDataTexture=!0,this.image={data:t,width:e,height:i},this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}};var Wo=class extends an{constructor(t,e,i,s=1){super(t,e,i),this.isInstancedBufferAttribute=!0,this.meshPerAttribute=s}copy(t){return super.copy(t),this.meshPerAttribute=t.meshPerAttribute,this}toJSON(){let t=super.toJSON();return t.meshPerAttribute=this.meshPerAttribute,t.isInstancedBufferAttribute=!0,t}},vr=new pe,Yd=new pe,bo=[],Zd=new jn,nv=new pe,ia=new Ot,sa=new Ns,xa=class extends Ot{constructor(t,e,i){super(t,e),this.isInstancedMesh=!0,this.instanceMatrix=new Wo(new Float32Array(i*16),16),this.instanceColor=null,this.morphTexture=null,this.count=i,this.boundingBox=null,this.boundingSphere=null;for(let s=0;s<i;s++)this.setMatrixAt(s,nv)}computeBoundingBox(){let t=this.geometry,e=this.count;this.boundingBox===null&&(this.boundingBox=new jn),t.boundingBox===null&&t.computeBoundingBox(),this.boundingBox.makeEmpty();for(let i=0;i<e;i++)this.getMatrixAt(i,vr),Zd.copy(t.boundingBox).applyMatrix4(vr),this.boundingBox.union(Zd)}computeBoundingSphere(){let t=this.geometry,e=this.count;this.boundingSphere===null&&(this.boundingSphere=new Ns),t.boundingSphere===null&&t.computeBoundingSphere(),this.boundingSphere.makeEmpty();for(let i=0;i<e;i++)this.getMatrixAt(i,vr),sa.copy(t.boundingSphere).applyMatrix4(vr),this.boundingSphere.union(sa)}copy(t,e){return super.copy(t,e),this.instanceMatrix.copy(t.instanceMatrix),t.morphTexture!==null&&(this.morphTexture=t.morphTexture.clone()),t.instanceColor!==null&&(this.instanceColor=t.instanceColor.clone()),this.count=t.count,t.boundingBox!==null&&(this.boundingBox=t.boundingBox.clone()),t.boundingSphere!==null&&(this.boundingSphere=t.boundingSphere.clone()),this}getColorAt(t,e){e.fromArray(this.instanceColor.array,t*3)}getMatrixAt(t,e){e.fromArray(this.instanceMatrix.array,t*16)}getMorphAt(t,e){let i=e.morphTargetInfluences,s=this.morphTexture.source.data.data,r=i.length+1,a=t*r+1;for(let o=0;o<i.length;o++)i[o]=s[a+o]}raycast(t,e){let i=this.matrixWorld,s=this.count;if(ia.geometry=this.geometry,ia.material=this.material,ia.material!==void 0&&(this.boundingSphere===null&&this.computeBoundingSphere(),sa.copy(this.boundingSphere),sa.applyMatrix4(i),t.ray.intersectsSphere(sa)!==!1))for(let r=0;r<s;r++){this.getMatrixAt(r,vr),Yd.multiplyMatrices(i,vr),ia.matrixWorld=Yd,ia.raycast(t,bo);for(let a=0,o=bo.length;a<o;a++){let l=bo[a];l.instanceId=r,l.object=this,e.push(l)}bo.length=0}}setColorAt(t,e){this.instanceColor===null&&(this.instanceColor=new Wo(new Float32Array(this.instanceMatrix.count*3).fill(1),3)),e.toArray(this.instanceColor.array,t*3)}setMatrixAt(t,e){e.toArray(this.instanceMatrix.array,t*16)}setMorphAt(t,e){let i=e.morphTargetInfluences,s=i.length+1;this.morphTexture===null&&(this.morphTexture=new Th(new Float32Array(s*this.count),s,this.count,su,Kn));let r=this.morphTexture.source.data.data,a=0;for(let c=0;c<i.length;c++)a+=i[c];let o=this.geometry.morphTargetsRelative?1:1-a,l=s*t;r[l]=o,r.set(i,l+1)}updateMorphTargets(){}dispose(){return this.dispatchEvent({type:"dispose"}),this.morphTexture!==null&&(this.morphTexture.dispose(),this.morphTexture=null),this}};var qo=class extends on{constructor(t,e,i,s,r,a,o,l,c){super(t,e,i,s,r,a,o,l,c),this.isCanvasTexture=!0,this.needsUpdate=!0}},Pn=class{constructor(){this.type="Curve",this.arcLengthDivisions=200}getPoint(){return console.warn("THREE.Curve: .getPoint() not implemented."),null}getPointAt(t,e){let i=this.getUtoTmapping(t);return this.getPoint(i,e)}getPoints(t=5){let e=[];for(let i=0;i<=t;i++)e.push(this.getPoint(i/t));return e}getSpacedPoints(t=5){let e=[];for(let i=0;i<=t;i++)e.push(this.getPointAt(i/t));return e}getLength(){let t=this.getLengths();return t[t.length-1]}getLengths(t=this.arcLengthDivisions){if(this.cacheArcLengths&&this.cacheArcLengths.length===t+1&&!this.needsUpdate)return this.cacheArcLengths;this.needsUpdate=!1;let e=[],i,s=this.getPoint(0),r=0;e.push(0);for(let a=1;a<=t;a++)i=this.getPoint(a/t),r+=i.distanceTo(s),e.push(r),s=i;return this.cacheArcLengths=e,e}updateArcLengths(){this.needsUpdate=!0,this.getLengths()}getUtoTmapping(t,e){let i=this.getLengths(),s=0,r=i.length,a;e?a=e:a=t*i[r-1];let o=0,l=r-1,c;for(;o<=l;)if(s=Math.floor(o+(l-o)/2),c=i[s]-a,c<0)o=s+1;else if(c>0)l=s-1;else{l=s;break}if(s=l,i[s]===a)return s/(r-1);let h=i[s],d=i[s+1]-h,f=(a-h)/d;return(s+f)/(r-1)}getTangent(t,e){let s=t-1e-4,r=t+1e-4;s<0&&(s=0),r>1&&(r=1);let a=this.getPoint(s),o=this.getPoint(r),l=e||(a.isVector2?new j:new I);return l.copy(o).sub(a).normalize(),l}getTangentAt(t,e){let i=this.getUtoTmapping(t);return this.getTangent(i,e)}computeFrenetFrames(t,e){let i=new I,s=[],r=[],a=[],o=new I,l=new pe;for(let f=0;f<=t;f++){let g=f/t;s[f]=this.getTangentAt(g,new I)}r[0]=new I,a[0]=new I;let c=Number.MAX_VALUE,h=Math.abs(s[0].x),u=Math.abs(s[0].y),d=Math.abs(s[0].z);h<=c&&(c=h,i.set(1,0,0)),u<=c&&(c=u,i.set(0,1,0)),d<=c&&i.set(0,0,1),o.crossVectors(s[0],i).normalize(),r[0].crossVectors(s[0],o),a[0].crossVectors(s[0],r[0]);for(let f=1;f<=t;f++){if(r[f]=r[f-1].clone(),a[f]=a[f-1].clone(),o.crossVectors(s[f-1],s[f]),o.length()>Number.EPSILON){o.normalize();let g=Math.acos(qe(s[f-1].dot(s[f]),-1,1));r[f].applyMatrix4(l.makeRotationAxis(o,g))}a[f].crossVectors(s[f],r[f])}if(e===!0){let f=Math.acos(qe(r[0].dot(r[t]),-1,1));f/=t,s[0].dot(o.crossVectors(r[0],r[t]))>0&&(f=-f);for(let g=1;g<=t;g++)r[g].applyMatrix4(l.makeRotationAxis(s[g],f*g)),a[g].crossVectors(s[g],r[g])}return{tangents:s,normals:r,binormals:a}}clone(){return new this.constructor().copy(this)}copy(t){return this.arcLengthDivisions=t.arcLengthDivisions,this}toJSON(){let t={metadata:{version:4.6,type:"Curve",generator:"Curve.toJSON"}};return t.arcLengthDivisions=this.arcLengthDivisions,t.type=this.type,t}fromJSON(t){return this.arcLengthDivisions=t.arcLengthDivisions,this}},va=class extends Pn{constructor(t=0,e=0,i=1,s=1,r=0,a=Math.PI*2,o=!1,l=0){super(),this.isEllipseCurve=!0,this.type="EllipseCurve",this.aX=t,this.aY=e,this.xRadius=i,this.yRadius=s,this.aStartAngle=r,this.aEndAngle=a,this.aClockwise=o,this.aRotation=l}getPoint(t,e=new j){let i=e,s=Math.PI*2,r=this.aEndAngle-this.aStartAngle,a=Math.abs(r)<Number.EPSILON;for(;r<0;)r+=s;for(;r>s;)r-=s;r<Number.EPSILON&&(a?r=0:r=s),this.aClockwise===!0&&!a&&(r===s?r=-s:r=r-s);let o=this.aStartAngle+t*r,l=this.aX+this.xRadius*Math.cos(o),c=this.aY+this.yRadius*Math.sin(o);if(this.aRotation!==0){let h=Math.cos(this.aRotation),u=Math.sin(this.aRotation),d=l-this.aX,f=c-this.aY;l=d*h-f*u+this.aX,c=d*u+f*h+this.aY}return i.set(l,c)}copy(t){return super.copy(t),this.aX=t.aX,this.aY=t.aY,this.xRadius=t.xRadius,this.yRadius=t.yRadius,this.aStartAngle=t.aStartAngle,this.aEndAngle=t.aEndAngle,this.aClockwise=t.aClockwise,this.aRotation=t.aRotation,this}toJSON(){let t=super.toJSON();return t.aX=this.aX,t.aY=this.aY,t.xRadius=this.xRadius,t.yRadius=this.yRadius,t.aStartAngle=this.aStartAngle,t.aEndAngle=this.aEndAngle,t.aClockwise=this.aClockwise,t.aRotation=this.aRotation,t}fromJSON(t){return super.fromJSON(t),this.aX=t.aX,this.aY=t.aY,this.xRadius=t.xRadius,this.yRadius=t.yRadius,this.aStartAngle=t.aStartAngle,this.aEndAngle=t.aEndAngle,this.aClockwise=t.aClockwise,this.aRotation=t.aRotation,this}},Ah=class extends va{constructor(t,e,i,s,r,a){super(t,e,i,i,s,r,a),this.isArcCurve=!0,this.type="ArcCurve"}};function cu(){let n=0,t=0,e=0,i=0;function s(r,a,o,l){n=r,t=o,e=-3*r+3*a-2*o-l,i=2*r-2*a+o+l}return{initCatmullRom:function(r,a,o,l,c){s(a,o,c*(o-r),c*(l-a))},initNonuniformCatmullRom:function(r,a,o,l,c,h,u){let d=(a-r)/c-(o-r)/(c+h)+(o-a)/h,f=(o-a)/h-(l-a)/(h+u)+(l-o)/u;d*=h,f*=h,s(a,o,d,f)},calc:function(r){let a=r*r,o=a*r;return n+t*r+e*a+i*o}}}var Mo=new I,gc=new cu,xc=new cu,vc=new cu,Ch=class extends Pn{constructor(t=[],e=!1,i="centripetal",s=.5){super(),this.isCatmullRomCurve3=!0,this.type="CatmullRomCurve3",this.points=t,this.closed=e,this.curveType=i,this.tension=s}getPoint(t,e=new I){let i=e,s=this.points,r=s.length,a=(r-(this.closed?0:1))*t,o=Math.floor(a),l=a-o;this.closed?o+=o>0?0:(Math.floor(Math.abs(o)/r)+1)*r:l===0&&o===r-1&&(o=r-2,l=1);let c,h;this.closed||o>0?c=s[(o-1)%r]:(Mo.subVectors(s[0],s[1]).add(s[0]),c=Mo);let u=s[o%r],d=s[(o+1)%r];if(this.closed||o+2<r?h=s[(o+2)%r]:(Mo.subVectors(s[r-1],s[r-2]).add(s[r-1]),h=Mo),this.curveType==="centripetal"||this.curveType==="chordal"){let f=this.curveType==="chordal"?.5:.25,g=Math.pow(c.distanceToSquared(u),f),x=Math.pow(u.distanceToSquared(d),f),m=Math.pow(d.distanceToSquared(h),f);x<1e-4&&(x=1),g<1e-4&&(g=x),m<1e-4&&(m=x),gc.initNonuniformCatmullRom(c.x,u.x,d.x,h.x,g,x,m),xc.initNonuniformCatmullRom(c.y,u.y,d.y,h.y,g,x,m),vc.initNonuniformCatmullRom(c.z,u.z,d.z,h.z,g,x,m)}else this.curveType==="catmullrom"&&(gc.initCatmullRom(c.x,u.x,d.x,h.x,this.tension),xc.initCatmullRom(c.y,u.y,d.y,h.y,this.tension),vc.initCatmullRom(c.z,u.z,d.z,h.z,this.tension));return i.set(gc.calc(l),xc.calc(l),vc.calc(l)),i}copy(t){super.copy(t),this.points=[];for(let e=0,i=t.points.length;e<i;e++){let s=t.points[e];this.points.push(s.clone())}return this.closed=t.closed,this.curveType=t.curveType,this.tension=t.tension,this}toJSON(){let t=super.toJSON();t.points=[];for(let e=0,i=this.points.length;e<i;e++){let s=this.points[e];t.points.push(s.toArray())}return t.closed=this.closed,t.curveType=this.curveType,t.tension=this.tension,t}fromJSON(t){super.fromJSON(t),this.points=[];for(let e=0,i=t.points.length;e<i;e++){let s=t.points[e];this.points.push(new I().fromArray(s))}return this.closed=t.closed,this.curveType=t.curveType,this.tension=t.tension,this}};function Jd(n,t,e,i,s){let r=(i-t)*.5,a=(s-e)*.5,o=n*n,l=n*o;return(2*e-2*i+r+a)*l+(-3*e+3*i-2*r-a)*o+r*n+e}function iv(n,t){let e=1-n;return e*e*t}function sv(n,t){return 2*(1-n)*n*t}function rv(n,t){return n*n*t}function ca(n,t,e,i){return iv(n,t)+sv(n,e)+rv(n,i)}function av(n,t){let e=1-n;return e*e*e*t}function ov(n,t){let e=1-n;return 3*e*e*n*t}function lv(n,t){return 3*(1-n)*n*n*t}function cv(n,t){return n*n*n*t}function ha(n,t,e,i,s){return av(n,t)+ov(n,e)+lv(n,i)+cv(n,s)}var Xo=class extends Pn{constructor(t=new j,e=new j,i=new j,s=new j){super(),this.isCubicBezierCurve=!0,this.type="CubicBezierCurve",this.v0=t,this.v1=e,this.v2=i,this.v3=s}getPoint(t,e=new j){let i=e,s=this.v0,r=this.v1,a=this.v2,o=this.v3;return i.set(ha(t,s.x,r.x,a.x,o.x),ha(t,s.y,r.y,a.y,o.y)),i}copy(t){return super.copy(t),this.v0.copy(t.v0),this.v1.copy(t.v1),this.v2.copy(t.v2),this.v3.copy(t.v3),this}toJSON(){let t=super.toJSON();return t.v0=this.v0.toArray(),t.v1=this.v1.toArray(),t.v2=this.v2.toArray(),t.v3=this.v3.toArray(),t}fromJSON(t){return super.fromJSON(t),this.v0.fromArray(t.v0),this.v1.fromArray(t.v1),this.v2.fromArray(t.v2),this.v3.fromArray(t.v3),this}},Rh=class extends Pn{constructor(t=new I,e=new I,i=new I,s=new I){super(),this.isCubicBezierCurve3=!0,this.type="CubicBezierCurve3",this.v0=t,this.v1=e,this.v2=i,this.v3=s}getPoint(t,e=new I){let i=e,s=this.v0,r=this.v1,a=this.v2,o=this.v3;return i.set(ha(t,s.x,r.x,a.x,o.x),ha(t,s.y,r.y,a.y,o.y),ha(t,s.z,r.z,a.z,o.z)),i}copy(t){return super.copy(t),this.v0.copy(t.v0),this.v1.copy(t.v1),this.v2.copy(t.v2),this.v3.copy(t.v3),this}toJSON(){let t=super.toJSON();return t.v0=this.v0.toArray(),t.v1=this.v1.toArray(),t.v2=this.v2.toArray(),t.v3=this.v3.toArray(),t}fromJSON(t){return super.fromJSON(t),this.v0.fromArray(t.v0),this.v1.fromArray(t.v1),this.v2.fromArray(t.v2),this.v3.fromArray(t.v3),this}},$o=class extends Pn{constructor(t=new j,e=new j){super(),this.isLineCurve=!0,this.type="LineCurve",this.v1=t,this.v2=e}getPoint(t,e=new j){let i=e;return t===1?i.copy(this.v2):(i.copy(this.v2).sub(this.v1),i.multiplyScalar(t).add(this.v1)),i}getPointAt(t,e){return this.getPoint(t,e)}getTangent(t,e=new j){return e.subVectors(this.v2,this.v1).normalize()}getTangentAt(t,e){return this.getTangent(t,e)}copy(t){return super.copy(t),this.v1.copy(t.v1),this.v2.copy(t.v2),this}toJSON(){let t=super.toJSON();return t.v1=this.v1.toArray(),t.v2=this.v2.toArray(),t}fromJSON(t){return super.fromJSON(t),this.v1.fromArray(t.v1),this.v2.fromArray(t.v2),this}},Ph=class extends Pn{constructor(t=new I,e=new I){super(),this.isLineCurve3=!0,this.type="LineCurve3",this.v1=t,this.v2=e}getPoint(t,e=new I){let i=e;return t===1?i.copy(this.v2):(i.copy(this.v2).sub(this.v1),i.multiplyScalar(t).add(this.v1)),i}getPointAt(t,e){return this.getPoint(t,e)}getTangent(t,e=new I){return e.subVectors(this.v2,this.v1).normalize()}getTangentAt(t,e){return this.getTangent(t,e)}copy(t){return super.copy(t),this.v1.copy(t.v1),this.v2.copy(t.v2),this}toJSON(){let t=super.toJSON();return t.v1=this.v1.toArray(),t.v2=this.v2.toArray(),t}fromJSON(t){return super.fromJSON(t),this.v1.fromArray(t.v1),this.v2.fromArray(t.v2),this}},Yo=class extends Pn{constructor(t=new j,e=new j,i=new j){super(),this.isQuadraticBezierCurve=!0,this.type="QuadraticBezierCurve",this.v0=t,this.v1=e,this.v2=i}getPoint(t,e=new j){let i=e,s=this.v0,r=this.v1,a=this.v2;return i.set(ca(t,s.x,r.x,a.x),ca(t,s.y,r.y,a.y)),i}copy(t){return super.copy(t),this.v0.copy(t.v0),this.v1.copy(t.v1),this.v2.copy(t.v2),this}toJSON(){let t=super.toJSON();return t.v0=this.v0.toArray(),t.v1=this.v1.toArray(),t.v2=this.v2.toArray(),t}fromJSON(t){return super.fromJSON(t),this.v0.fromArray(t.v0),this.v1.fromArray(t.v1),this.v2.fromArray(t.v2),this}},Ih=class extends Pn{constructor(t=new I,e=new I,i=new I){super(),this.isQuadraticBezierCurve3=!0,this.type="QuadraticBezierCurve3",this.v0=t,this.v1=e,this.v2=i}getPoint(t,e=new I){let i=e,s=this.v0,r=this.v1,a=this.v2;return i.set(ca(t,s.x,r.x,a.x),ca(t,s.y,r.y,a.y),ca(t,s.z,r.z,a.z)),i}copy(t){return super.copy(t),this.v0.copy(t.v0),this.v1.copy(t.v1),this.v2.copy(t.v2),this}toJSON(){let t=super.toJSON();return t.v0=this.v0.toArray(),t.v1=this.v1.toArray(),t.v2=this.v2.toArray(),t}fromJSON(t){return super.fromJSON(t),this.v0.fromArray(t.v0),this.v1.fromArray(t.v1),this.v2.fromArray(t.v2),this}},Zo=class extends Pn{constructor(t=[]){super(),this.isSplineCurve=!0,this.type="SplineCurve",this.points=t}getPoint(t,e=new j){let i=e,s=this.points,r=(s.length-1)*t,a=Math.floor(r),o=r-a,l=s[a===0?a:a-1],c=s[a],h=s[a>s.length-2?s.length-1:a+1],u=s[a>s.length-3?s.length-1:a+2];return i.set(Jd(o,l.x,c.x,h.x,u.x),Jd(o,l.y,c.y,h.y,u.y)),i}copy(t){super.copy(t),this.points=[];for(let e=0,i=t.points.length;e<i;e++){let s=t.points[e];this.points.push(s.clone())}return this}toJSON(){let t=super.toJSON();t.points=[];for(let e=0,i=this.points.length;e<i;e++){let s=this.points[e];t.points.push(s.toArray())}return t}fromJSON(t){super.fromJSON(t),this.points=[];for(let e=0,i=t.points.length;e<i;e++){let s=t.points[e];this.points.push(new j().fromArray(s))}return this}},Fh=Object.freeze({__proto__:null,ArcCurve:Ah,CatmullRomCurve3:Ch,CubicBezierCurve:Xo,CubicBezierCurve3:Rh,EllipseCurve:va,LineCurve:$o,LineCurve3:Ph,QuadraticBezierCurve:Yo,QuadraticBezierCurve3:Ih,SplineCurve:Zo}),Dh=class extends Pn{constructor(){super(),this.type="CurvePath",this.curves=[],this.autoClose=!1}add(t){this.curves.push(t)}closePath(){let t=this.curves[0].getPoint(0),e=this.curves[this.curves.length-1].getPoint(1);if(!t.equals(e)){let i=t.isVector2===!0?"LineCurve":"LineCurve3";this.curves.push(new Fh[i](e,t))}return this}getPoint(t,e){let i=t*this.getLength(),s=this.getCurveLengths(),r=0;for(;r<s.length;){if(s[r]>=i){let a=s[r]-i,o=this.curves[r],l=o.getLength(),c=l===0?0:1-a/l;return o.getPointAt(c,e)}r++}return null}getLength(){let t=this.getCurveLengths();return t[t.length-1]}updateArcLengths(){this.needsUpdate=!0,this.cacheLengths=null,this.getCurveLengths()}getCurveLengths(){if(this.cacheLengths&&this.cacheLengths.length===this.curves.length)return this.cacheLengths;let t=[],e=0;for(let i=0,s=this.curves.length;i<s;i++)e+=this.curves[i].getLength(),t.push(e);return this.cacheLengths=t,t}getSpacedPoints(t=40){let e=[];for(let i=0;i<=t;i++)e.push(this.getPoint(i/t));return this.autoClose&&e.push(e[0]),e}getPoints(t=12){let e=[],i;for(let s=0,r=this.curves;s<r.length;s++){let a=r[s],o=a.isEllipseCurve?t*2:a.isLineCurve||a.isLineCurve3?1:a.isSplineCurve?t*a.points.length:t,l=a.getPoints(o);for(let c=0;c<l.length;c++){let h=l[c];i&&i.equals(h)||(e.push(h),i=h)}}return this.autoClose&&e.length>1&&!e[e.length-1].equals(e[0])&&e.push(e[0]),e}copy(t){super.copy(t),this.curves=[];for(let e=0,i=t.curves.length;e<i;e++){let s=t.curves[e];this.curves.push(s.clone())}return this.autoClose=t.autoClose,this}toJSON(){let t=super.toJSON();t.autoClose=this.autoClose,t.curves=[];for(let e=0,i=this.curves.length;e<i;e++){let s=this.curves[e];t.curves.push(s.toJSON())}return t}fromJSON(t){super.fromJSON(t),this.autoClose=t.autoClose,this.curves=[];for(let e=0,i=t.curves.length;e<i;e++){let s=t.curves[e];this.curves.push(new Fh[s.type]().fromJSON(s))}return this}},_n=class extends Dh{constructor(t){super(),this.type="Path",this.currentPoint=new j,t&&this.setFromPoints(t)}setFromPoints(t){this.moveTo(t[0].x,t[0].y);for(let e=1,i=t.length;e<i;e++)this.lineTo(t[e].x,t[e].y);return this}moveTo(t,e){return this.currentPoint.set(t,e),this}lineTo(t,e){let i=new $o(this.currentPoint.clone(),new j(t,e));return this.curves.push(i),this.currentPoint.set(t,e),this}quadraticCurveTo(t,e,i,s){let r=new Yo(this.currentPoint.clone(),new j(t,e),new j(i,s));return this.curves.push(r),this.currentPoint.set(i,s),this}bezierCurveTo(t,e,i,s,r,a){let o=new Xo(this.currentPoint.clone(),new j(t,e),new j(i,s),new j(r,a));return this.curves.push(o),this.currentPoint.set(r,a),this}splineThru(t){let e=[this.currentPoint.clone()].concat(t),i=new Zo(e);return this.curves.push(i),this.currentPoint.copy(t[t.length-1]),this}arc(t,e,i,s,r,a){let o=this.currentPoint.x,l=this.currentPoint.y;return this.absarc(t+o,e+l,i,s,r,a),this}absarc(t,e,i,s,r,a){return this.absellipse(t,e,i,i,s,r,a),this}ellipse(t,e,i,s,r,a,o,l){let c=this.currentPoint.x,h=this.currentPoint.y;return this.absellipse(t+c,e+h,i,s,r,a,o,l),this}absellipse(t,e,i,s,r,a,o,l){let c=new va(t,e,i,s,r,a,o,l);if(this.curves.length>0){let u=c.getPoint(0);u.equals(this.currentPoint)||this.lineTo(u.x,u.y)}this.curves.push(c);let h=c.getPoint(1);return this.currentPoint.copy(h),this}copy(t){return super.copy(t),this.currentPoint.copy(t.currentPoint),this}toJSON(){let t=super.toJSON();return t.currentPoint=this.currentPoint.toArray(),t}fromJSON(t){return super.fromJSON(t),this.currentPoint.fromArray(t.currentPoint),this}};var ya=class n extends tn{constructor(t=1,e=32,i=0,s=Math.PI*2){super(),this.type="CircleGeometry",this.parameters={radius:t,segments:e,thetaStart:i,thetaLength:s},e=Math.max(3,e);let r=[],a=[],o=[],l=[],c=new I,h=new j;a.push(0,0,0),o.push(0,0,1),l.push(.5,.5);for(let u=0,d=3;u<=e;u++,d+=3){let f=i+u/e*s;c.x=t*Math.cos(f),c.y=t*Math.sin(f),a.push(c.x,c.y,c.z),o.push(0,0,1),h.x=(a[d]/t+1)/2,h.y=(a[d+1]/t+1)/2,l.push(h.x,h.y)}for(let u=1;u<=e;u++)r.push(u,u+1,0);this.setIndex(r),this.setAttribute("position",new _e(a,3)),this.setAttribute("normal",new _e(o,3)),this.setAttribute("uv",new _e(l,2))}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new n(t.radius,t.segments,t.thetaStart,t.thetaLength)}},ni=class n extends tn{constructor(t=1,e=1,i=1,s=32,r=1,a=!1,o=0,l=Math.PI*2){super(),this.type="CylinderGeometry",this.parameters={radiusTop:t,radiusBottom:e,height:i,radialSegments:s,heightSegments:r,openEnded:a,thetaStart:o,thetaLength:l};let c=this;s=Math.floor(s),r=Math.floor(r);let h=[],u=[],d=[],f=[],g=0,x=[],m=i/2,p=0;T(),a===!1&&(t>0&&M(!0),e>0&&M(!1)),this.setIndex(h),this.setAttribute("position",new _e(u,3)),this.setAttribute("normal",new _e(d,3)),this.setAttribute("uv",new _e(f,2));function T(){let y=new I,D=new I,S=0,C=(e-t)/i;for(let P=0;P<=r;P++){let w=[],_=P/r,R=_*(e-t)+t;for(let G=0;G<=s;G++){let N=G/s,z=N*l+o,J=Math.sin(z),V=Math.cos(z);D.x=R*J,D.y=-_*i+m,D.z=R*V,u.push(D.x,D.y,D.z),y.set(J,C,V).normalize(),d.push(y.x,y.y,y.z),f.push(N,1-_),w.push(g++)}x.push(w)}for(let P=0;P<s;P++)for(let w=0;w<r;w++){let _=x[w][P],R=x[w+1][P],G=x[w+1][P+1],N=x[w][P+1];(t>0||w!==0)&&(h.push(_,R,N),S+=3),(e>0||w!==r-1)&&(h.push(R,G,N),S+=3)}c.addGroup(p,S,0),p+=S}function M(y){let D=g,S=new j,C=new I,P=0,w=y===!0?t:e,_=y===!0?1:-1;for(let G=1;G<=s;G++)u.push(0,m*_,0),d.push(0,_,0),f.push(.5,.5),g++;let R=g;for(let G=0;G<=s;G++){let z=G/s*l+o,J=Math.cos(z),V=Math.sin(z);C.x=w*V,C.y=m*_,C.z=w*J,u.push(C.x,C.y,C.z),d.push(0,_,0),S.x=J*.5+.5,S.y=V*.5*_+.5,f.push(S.x,S.y),g++}for(let G=0;G<s;G++){let N=D+G,z=R+G;y===!0?h.push(z,z+1,N):h.push(z+1,z,N),P+=3}c.addGroup(p,P,y===!0?1:2),p+=P}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new n(t.radiusTop,t.radiusBottom,t.height,t.radialSegments,t.heightSegments,t.openEnded,t.thetaStart,t.thetaLength)}};var Xe=class extends _n{constructor(t){super(t),this.uuid=Ei(),this.type="Shape",this.holes=[]}getPointsHoles(t){let e=[];for(let i=0,s=this.holes.length;i<s;i++)e[i]=this.holes[i].getPoints(t);return e}extractPoints(t){return{shape:this.getPoints(t),holes:this.getPointsHoles(t)}}copy(t){super.copy(t),this.holes=[];for(let e=0,i=t.holes.length;e<i;e++){let s=t.holes[e];this.holes.push(s.clone())}return this}toJSON(){let t=super.toJSON();t.uuid=this.uuid,t.holes=[];for(let e=0,i=this.holes.length;e<i;e++){let s=this.holes[e];t.holes.push(s.toJSON())}return t}fromJSON(t){super.fromJSON(t),this.uuid=t.uuid,this.holes=[];for(let e=0,i=t.holes.length;e<i;e++){let s=t.holes[e];this.holes.push(new _n().fromJSON(s))}return this}},hv={triangulate:function(n,t,e=2){let i=t&&t.length,s=i?t[0]*e:n.length,r=Rf(n,0,s,e,!0),a=[];if(!r||r.next===r.prev)return a;let o,l,c,h,u,d,f;if(i&&(r=mv(n,t,r,e)),n.length>80*e){o=c=n[0],l=h=n[1];for(let g=e;g<s;g+=e)u=n[g],d=n[g+1],u<o&&(o=u),d<l&&(l=d),u>c&&(c=u),d>h&&(h=d);f=Math.max(c-o,h-l),f=f!==0?32767/f:0}return _a(r,a,e,o,l,f,0),a}};function Rf(n,t,e,i,s){let r,a;if(s===Tv(n,t,e,i)>0)for(r=t;r<e;r+=i)a=Kd(r,n[r],n[r+1],a);else for(r=e-i;r>=t;r-=i)a=Kd(r,n[r],n[r+1],a);return a&&ol(a,a.next)&&(Ma(a),a=a.next),a}function Bs(n,t){if(!n)return n;t||(t=n);let e=n,i;do if(i=!1,!e.steiner&&(ol(e,e.next)||Ae(e.prev,e,e.next)===0)){if(Ma(e),e=t=e.prev,e===e.next)break;i=!0}else e=e.next;while(i||e!==t);return t}function _a(n,t,e,i,s,r,a){if(!n)return;!a&&r&&_v(n,i,s,r);let o=n,l,c;for(;n.prev!==n.next;){if(l=n.prev,c=n.next,r?dv(n,i,s,r):uv(n)){t.push(l.i/e|0),t.push(n.i/e|0),t.push(c.i/e|0),Ma(n),n=c.next,o=c.next;continue}if(n=c,n===o){a?a===1?(n=fv(Bs(n),t,e),_a(n,t,e,i,s,r,2)):a===2&&pv(n,t,e,i,s,r):_a(Bs(n),t,e,i,s,r,1);break}}}function uv(n){let t=n.prev,e=n,i=n.next;if(Ae(t,e,i)>=0)return!1;let s=t.x,r=e.x,a=i.x,o=t.y,l=e.y,c=i.y,h=s<r?s<a?s:a:r<a?r:a,u=o<l?o<c?o:c:l<c?l:c,d=s>r?s>a?s:a:r>a?r:a,f=o>l?o>c?o:c:l>c?l:c,g=i.next;for(;g!==t;){if(g.x>=h&&g.x<=d&&g.y>=u&&g.y<=f&&_r(s,o,r,l,a,c,g.x,g.y)&&Ae(g.prev,g,g.next)>=0)return!1;g=g.next}return!0}function dv(n,t,e,i){let s=n.prev,r=n,a=n.next;if(Ae(s,r,a)>=0)return!1;let o=s.x,l=r.x,c=a.x,h=s.y,u=r.y,d=a.y,f=o<l?o<c?o:c:l<c?l:c,g=h<u?h<d?h:d:u<d?u:d,x=o>l?o>c?o:c:l>c?l:c,m=h>u?h>d?h:d:u>d?u:d,p=kh(f,g,t,e,i),T=kh(x,m,t,e,i),M=n.prevZ,y=n.nextZ;for(;M&&M.z>=p&&y&&y.z<=T;){if(M.x>=f&&M.x<=x&&M.y>=g&&M.y<=m&&M!==s&&M!==a&&_r(o,h,l,u,c,d,M.x,M.y)&&Ae(M.prev,M,M.next)>=0||(M=M.prevZ,y.x>=f&&y.x<=x&&y.y>=g&&y.y<=m&&y!==s&&y!==a&&_r(o,h,l,u,c,d,y.x,y.y)&&Ae(y.prev,y,y.next)>=0))return!1;y=y.nextZ}for(;M&&M.z>=p;){if(M.x>=f&&M.x<=x&&M.y>=g&&M.y<=m&&M!==s&&M!==a&&_r(o,h,l,u,c,d,M.x,M.y)&&Ae(M.prev,M,M.next)>=0)return!1;M=M.prevZ}for(;y&&y.z<=T;){if(y.x>=f&&y.x<=x&&y.y>=g&&y.y<=m&&y!==s&&y!==a&&_r(o,h,l,u,c,d,y.x,y.y)&&Ae(y.prev,y,y.next)>=0)return!1;y=y.nextZ}return!0}function fv(n,t,e){let i=n;do{let s=i.prev,r=i.next.next;!ol(s,r)&&Pf(s,i,i.next,r)&&ba(s,r)&&ba(r,s)&&(t.push(s.i/e|0),t.push(i.i/e|0),t.push(r.i/e|0),Ma(i),Ma(i.next),i=n=r),i=i.next}while(i!==n);return Bs(i)}function pv(n,t,e,i,s,r){let a=n;do{let o=a.next.next;for(;o!==a.prev;){if(a.i!==o.i&&Sv(a,o)){let l=If(a,o);a=Bs(a,a.next),l=Bs(l,l.next),_a(a,t,e,i,s,r,0),_a(l,t,e,i,s,r,0);return}o=o.next}a=a.next}while(a!==n)}function mv(n,t,e,i){let s=[],r,a,o,l,c;for(r=0,a=t.length;r<a;r++)o=t[r]*i,l=r<a-1?t[r+1]*i:n.length,c=Rf(n,o,l,i,!1),c===c.next&&(c.steiner=!0),s.push(Mv(c));for(s.sort(gv),r=0;r<s.length;r++)e=xv(s[r],e);return e}function gv(n,t){return n.x-t.x}function xv(n,t){let e=vv(n,t);if(!e)return t;let i=If(e,n);return Bs(i,i.next),Bs(e,e.next)}function vv(n,t){let e=t,i=-1/0,s,r=n.x,a=n.y;do{if(a<=e.y&&a>=e.next.y&&e.next.y!==e.y){let d=e.x+(a-e.y)*(e.next.x-e.x)/(e.next.y-e.y);if(d<=r&&d>i&&(i=d,s=e.x<e.next.x?e:e.next,d===r))return s}e=e.next}while(e!==t);if(!s)return null;let o=s,l=s.x,c=s.y,h=1/0,u;e=s;do r>=e.x&&e.x>=l&&r!==e.x&&_r(a<c?r:i,a,l,c,a<c?i:r,a,e.x,e.y)&&(u=Math.abs(a-e.y)/(r-e.x),ba(e,n)&&(u<h||u===h&&(e.x>s.x||e.x===s.x&&yv(s,e)))&&(s=e,h=u)),e=e.next;while(e!==o);return s}function yv(n,t){return Ae(n.prev,n,t.prev)<0&&Ae(t.next,n,n.next)<0}function _v(n,t,e,i){let s=n;do s.z===0&&(s.z=kh(s.x,s.y,t,e,i)),s.prevZ=s.prev,s.nextZ=s.next,s=s.next;while(s!==n);s.prevZ.nextZ=null,s.prevZ=null,bv(s)}function bv(n){let t,e,i,s,r,a,o,l,c=1;do{for(e=n,n=null,r=null,a=0;e;){for(a++,i=e,o=0,t=0;t<c&&(o++,i=i.nextZ,!!i);t++);for(l=c;o>0||l>0&&i;)o!==0&&(l===0||!i||e.z<=i.z)?(s=e,e=e.nextZ,o--):(s=i,i=i.nextZ,l--),r?r.nextZ=s:n=s,s.prevZ=r,r=s;e=i}r.nextZ=null,c*=2}while(a>1);return n}function kh(n,t,e,i,s){return n=(n-e)*s|0,t=(t-i)*s|0,n=(n|n<<8)&16711935,n=(n|n<<4)&252645135,n=(n|n<<2)&858993459,n=(n|n<<1)&1431655765,t=(t|t<<8)&16711935,t=(t|t<<4)&252645135,t=(t|t<<2)&858993459,t=(t|t<<1)&1431655765,n|t<<1}function Mv(n){let t=n,e=n;do(t.x<e.x||t.x===e.x&&t.y<e.y)&&(e=t),t=t.next;while(t!==n);return e}function _r(n,t,e,i,s,r,a,o){return(s-a)*(t-o)>=(n-a)*(r-o)&&(n-a)*(i-o)>=(e-a)*(t-o)&&(e-a)*(r-o)>=(s-a)*(i-o)}function Sv(n,t){return n.next.i!==t.i&&n.prev.i!==t.i&&!wv(n,t)&&(ba(n,t)&&ba(t,n)&&Ev(n,t)&&(Ae(n.prev,n,t.prev)||Ae(n,t.prev,t))||ol(n,t)&&Ae(n.prev,n,n.next)>0&&Ae(t.prev,t,t.next)>0)}function Ae(n,t,e){return(t.y-n.y)*(e.x-t.x)-(t.x-n.x)*(e.y-t.y)}function ol(n,t){return n.x===t.x&&n.y===t.y}function Pf(n,t,e,i){let s=wo(Ae(n,t,e)),r=wo(Ae(n,t,i)),a=wo(Ae(e,i,n)),o=wo(Ae(e,i,t));return!!(s!==r&&a!==o||s===0&&So(n,e,t)||r===0&&So(n,i,t)||a===0&&So(e,n,i)||o===0&&So(e,t,i))}function So(n,t,e){return t.x<=Math.max(n.x,e.x)&&t.x>=Math.min(n.x,e.x)&&t.y<=Math.max(n.y,e.y)&&t.y>=Math.min(n.y,e.y)}function wo(n){return n>0?1:n<0?-1:0}function wv(n,t){let e=n;do{if(e.i!==n.i&&e.next.i!==n.i&&e.i!==t.i&&e.next.i!==t.i&&Pf(e,e.next,n,t))return!0;e=e.next}while(e!==n);return!1}function ba(n,t){return Ae(n.prev,n,n.next)<0?Ae(n,t,n.next)>=0&&Ae(n,n.prev,t)>=0:Ae(n,t,n.prev)<0||Ae(n,n.next,t)<0}function Ev(n,t){let e=n,i=!1,s=(n.x+t.x)/2,r=(n.y+t.y)/2;do e.y>r!=e.next.y>r&&e.next.y!==e.y&&s<(e.next.x-e.x)*(r-e.y)/(e.next.y-e.y)+e.x&&(i=!i),e=e.next;while(e!==n);return i}function If(n,t){let e=new Lh(n.i,n.x,n.y),i=new Lh(t.i,t.x,t.y),s=n.next,r=t.prev;return n.next=t,t.prev=n,e.next=s,s.prev=e,i.next=e,e.prev=i,r.next=i,i.prev=r,i}function Kd(n,t,e,i){let s=new Lh(n,t,e);return i?(s.next=i.next,s.prev=i,i.next.prev=s,i.next=s):(s.prev=s,s.next=s),s}function Ma(n){n.next.prev=n.prev,n.prev.next=n.next,n.prevZ&&(n.prevZ.nextZ=n.nextZ),n.nextZ&&(n.nextZ.prevZ=n.prevZ)}function Lh(n,t,e){this.i=n,this.x=t,this.y=e,this.prev=null,this.next=null,this.z=0,this.prevZ=null,this.nextZ=null,this.steiner=!1}function Tv(n,t,e,i){let s=0;for(let r=t,a=e-i;r<e;r+=i)s+=(n[a]-n[r])*(n[r+1]+n[a+1]),a=r;return s}var Ki=class n{static area(t){let e=t.length,i=0;for(let s=e-1,r=0;r<e;s=r++)i+=t[s].x*t[r].y-t[r].x*t[s].y;return i*.5}static isClockWise(t){return n.area(t)<0}static triangulateShape(t,e){let i=[],s=[],r=[];Qd(t),jd(i,t);let a=t.length;e.forEach(Qd);for(let l=0;l<e.length;l++)s.push(a),a+=e[l].length,jd(i,e[l]);let o=hv.triangulate(i,s);for(let l=0;l<o.length;l+=3)r.push(o.slice(l,l+3));return r}};function Qd(n){let t=n.length;t>2&&n[t-1].equals(n[0])&&n.pop()}function jd(n,t){for(let e=0;e<t.length;e++)n.push(t[e].x),n.push(t[e].y)}var ns=class n extends tn{constructor(t=new Xe([new j(.5,.5),new j(-.5,.5),new j(-.5,-.5),new j(.5,-.5)]),e={}){super(),this.type="ExtrudeGeometry",this.parameters={shapes:t,options:e},t=Array.isArray(t)?t:[t];let i=this,s=[],r=[];for(let o=0,l=t.length;o<l;o++){let c=t[o];a(c)}this.setAttribute("position",new _e(s,3)),this.setAttribute("uv",new _e(r,2)),this.computeVertexNormals();function a(o){let l=[],c=e.curveSegments!==void 0?e.curveSegments:12,h=e.steps!==void 0?e.steps:1,u=e.depth!==void 0?e.depth:1,d=e.bevelEnabled!==void 0?e.bevelEnabled:!0,f=e.bevelThickness!==void 0?e.bevelThickness:.2,g=e.bevelSize!==void 0?e.bevelSize:f-.1,x=e.bevelOffset!==void 0?e.bevelOffset:0,m=e.bevelSegments!==void 0?e.bevelSegments:3,p=e.extrudePath,T=e.UVGenerator!==void 0?e.UVGenerator:Av,M,y=!1,D,S,C,P;p&&(M=p.getSpacedPoints(h),y=!0,d=!1,D=p.computeFrenetFrames(h,!1),S=new I,C=new I,P=new I),d||(m=0,f=0,g=0,x=0);let w=o.extractPoints(c),_=w.shape,R=w.holes;if(!Ki.isClockWise(_)){_=_.reverse();for(let Q=0,st=R.length;Q<st;Q++){let A=R[Q];Ki.isClockWise(A)&&(R[Q]=A.reverse())}}let N=Ki.triangulateShape(_,R),z=_;for(let Q=0,st=R.length;Q<st;Q++){let A=R[Q];_=_.concat(A)}function J(Q,st,A){return st||console.error("THREE.ExtrudeGeometry: vec does not exist"),Q.clone().addScaledVector(st,A)}let V=_.length,it=N.length;function W(Q,st,A){let Ft,et,yt,lt=Q.x-st.x,zt=Q.y-st.y,xt=A.x-Q.x,E=A.y-Q.y,v=lt*lt+zt*zt,U=lt*E-zt*xt;if(Math.abs(U)>Number.EPSILON){let X=Math.sqrt(v),tt=Math.sqrt(xt*xt+E*E),$=st.x-zt/X,Et=st.y+lt/X,ut=A.x-E/tt,vt=A.y+xt/tt,ne=((ut-$)*E-(vt-Et)*xt)/(lt*E-zt*xt);Ft=$+lt*ne-Q.x,et=Et+zt*ne-Q.y;let nt=Ft*Ft+et*et;if(nt<=2)return new j(Ft,et);yt=Math.sqrt(nt/2)}else{let X=!1;lt>Number.EPSILON?xt>Number.EPSILON&&(X=!0):lt<-Number.EPSILON?xt<-Number.EPSILON&&(X=!0):Math.sign(zt)===Math.sign(E)&&(X=!0),X?(Ft=-zt,et=lt,yt=Math.sqrt(v)):(Ft=lt,et=zt,yt=Math.sqrt(v/2))}return new j(Ft/yt,et/yt)}let ct=[];for(let Q=0,st=z.length,A=st-1,Ft=Q+1;Q<st;Q++,A++,Ft++)A===st&&(A=0),Ft===st&&(Ft=0),ct[Q]=W(z[Q],z[A],z[Ft]);let gt=[],Mt,Yt=ct.concat();for(let Q=0,st=R.length;Q<st;Q++){let A=R[Q];Mt=[];for(let Ft=0,et=A.length,yt=et-1,lt=Ft+1;Ft<et;Ft++,yt++,lt++)yt===et&&(yt=0),lt===et&&(lt=0),Mt[Ft]=W(A[Ft],A[yt],A[lt]);gt.push(Mt),Yt=Yt.concat(Mt)}for(let Q=0;Q<m;Q++){let st=Q/m,A=f*Math.cos(st*Math.PI/2),Ft=g*Math.sin(st*Math.PI/2)+x;for(let et=0,yt=z.length;et<yt;et++){let lt=J(z[et],ct[et],Ft);ot(lt.x,lt.y,-A)}for(let et=0,yt=R.length;et<yt;et++){let lt=R[et];Mt=gt[et];for(let zt=0,xt=lt.length;zt<xt;zt++){let E=J(lt[zt],Mt[zt],Ft);ot(E.x,E.y,-A)}}}let he=g+x;for(let Q=0;Q<V;Q++){let st=d?J(_[Q],Yt[Q],he):_[Q];y?(C.copy(D.normals[0]).multiplyScalar(st.x),S.copy(D.binormals[0]).multiplyScalar(st.y),P.copy(M[0]).add(C).add(S),ot(P.x,P.y,P.z)):ot(st.x,st.y,0)}for(let Q=1;Q<=h;Q++)for(let st=0;st<V;st++){let A=d?J(_[st],Yt[st],he):_[st];y?(C.copy(D.normals[Q]).multiplyScalar(A.x),S.copy(D.binormals[Q]).multiplyScalar(A.y),P.copy(M[Q]).add(C).add(S),ot(P.x,P.y,P.z)):ot(A.x,A.y,u/h*Q)}for(let Q=m-1;Q>=0;Q--){let st=Q/m,A=f*Math.cos(st*Math.PI/2),Ft=g*Math.sin(st*Math.PI/2)+x;for(let et=0,yt=z.length;et<yt;et++){let lt=J(z[et],ct[et],Ft);ot(lt.x,lt.y,u+A)}for(let et=0,yt=R.length;et<yt;et++){let lt=R[et];Mt=gt[et];for(let zt=0,xt=lt.length;zt<xt;zt++){let E=J(lt[zt],Mt[zt],Ft);y?ot(E.x,E.y+M[h-1].y,M[h-1].x+A):ot(E.x,E.y,u+A)}}}Y(),rt();function Y(){let Q=s.length/3;if(d){let st=0,A=V*st;for(let Ft=0;Ft<it;Ft++){let et=N[Ft];Bt(et[2]+A,et[1]+A,et[0]+A)}st=h+m*2,A=V*st;for(let Ft=0;Ft<it;Ft++){let et=N[Ft];Bt(et[0]+A,et[1]+A,et[2]+A)}}else{for(let st=0;st<it;st++){let A=N[st];Bt(A[2],A[1],A[0])}for(let st=0;st<it;st++){let A=N[st];Bt(A[0]+V*h,A[1]+V*h,A[2]+V*h)}}i.addGroup(Q,s.length/3-Q,0)}function rt(){let Q=s.length/3,st=0;St(z,st),st+=z.length;for(let A=0,Ft=R.length;A<Ft;A++){let et=R[A];St(et,st),st+=et.length}i.addGroup(Q,s.length/3-Q,1)}function St(Q,st){let A=Q.length;for(;--A>=0;){let Ft=A,et=A-1;et<0&&(et=Q.length-1);for(let yt=0,lt=h+m*2;yt<lt;yt++){let zt=V*yt,xt=V*(yt+1),E=st+Ft+zt,v=st+et+zt,U=st+et+xt,X=st+Ft+xt;qt(E,v,U,X)}}}function ot(Q,st,A){l.push(Q),l.push(st),l.push(A)}function Bt(Q,st,A){Vt(Q),Vt(st),Vt(A);let Ft=s.length/3,et=T.generateTopUV(i,s,Ft-3,Ft-2,Ft-1);le(et[0]),le(et[1]),le(et[2])}function qt(Q,st,A,Ft){Vt(Q),Vt(st),Vt(Ft),Vt(st),Vt(A),Vt(Ft);let et=s.length/3,yt=T.generateSideWallUV(i,s,et-6,et-3,et-2,et-1);le(yt[0]),le(yt[1]),le(yt[3]),le(yt[1]),le(yt[2]),le(yt[3])}function Vt(Q){s.push(l[Q*3+0]),s.push(l[Q*3+1]),s.push(l[Q*3+2])}function le(Q){r.push(Q.x),r.push(Q.y)}}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}toJSON(){let t=super.toJSON(),e=this.parameters.shapes,i=this.parameters.options;return Cv(e,i,t)}static fromJSON(t,e){let i=[];for(let r=0,a=t.shapes.length;r<a;r++){let o=e[t.shapes[r]];i.push(o)}let s=t.options.extrudePath;return s!==void 0&&(t.options.extrudePath=new Fh[s.type]().fromJSON(s)),new n(i,t.options)}},Av={generateTopUV:function(n,t,e,i,s){let r=t[e*3],a=t[e*3+1],o=t[i*3],l=t[i*3+1],c=t[s*3],h=t[s*3+1];return[new j(r,a),new j(o,l),new j(c,h)]},generateSideWallUV:function(n,t,e,i,s,r){let a=t[e*3],o=t[e*3+1],l=t[e*3+2],c=t[i*3],h=t[i*3+1],u=t[i*3+2],d=t[s*3],f=t[s*3+1],g=t[s*3+2],x=t[r*3],m=t[r*3+1],p=t[r*3+2];return Math.abs(o-h)<Math.abs(a-c)?[new j(a,1-l),new j(c,1-u),new j(d,1-g),new j(x,1-p)]:[new j(o,1-l),new j(h,1-u),new j(f,1-g),new j(m,1-p)]}};function Cv(n,t,e){if(e.shapes=[],Array.isArray(n))for(let i=0,s=n.length;i<s;i++){let r=n[i];e.shapes.push(r.uuid)}else e.shapes.push(n.uuid);return e.options=Object.assign({},t),t.extrudePath!==void 0&&(e.options.extrudePath=t.extrudePath.toJSON()),e}var Dr=class n extends tn{constructor(t=new Xe([new j(0,.5),new j(-.5,-.5),new j(.5,-.5)]),e=12){super(),this.type="ShapeGeometry",this.parameters={shapes:t,curveSegments:e};let i=[],s=[],r=[],a=[],o=0,l=0;if(Array.isArray(t)===!1)c(t);else for(let h=0;h<t.length;h++)c(t[h]),this.addGroup(o,l,h),o+=l,l=0;this.setIndex(i),this.setAttribute("position",new _e(s,3)),this.setAttribute("normal",new _e(r,3)),this.setAttribute("uv",new _e(a,2));function c(h){let u=s.length/3,d=h.extractPoints(e),f=d.shape,g=d.holes;Ki.isClockWise(f)===!1&&(f=f.reverse());for(let m=0,p=g.length;m<p;m++){let T=g[m];Ki.isClockWise(T)===!0&&(g[m]=T.reverse())}let x=Ki.triangulateShape(f,g);for(let m=0,p=g.length;m<p;m++){let T=g[m];f=f.concat(T)}for(let m=0,p=f.length;m<p;m++){let T=f[m];s.push(T.x,T.y,0),r.push(0,0,1),a.push(T.x,T.y)}for(let m=0,p=x.length;m<p;m++){let T=x[m],M=T[0]+u,y=T[1]+u,D=T[2]+u;i.push(M,y,D),l+=3}}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}toJSON(){let t=super.toJSON(),e=this.parameters.shapes;return Rv(e,t)}static fromJSON(t,e){let i=[];for(let s=0,r=t.shapes.length;s<r;s++){let a=e[t.shapes[s]];i.push(a)}return new n(i,t.curveSegments)}};function Rv(n,t){if(t.shapes=[],Array.isArray(n))for(let e=0,i=n.length;e<i;e++){let s=n[e];t.shapes.push(s.uuid)}else t.shapes.push(n.uuid);return t}var is=class n extends tn{constructor(t=1,e=32,i=16,s=0,r=Math.PI*2,a=0,o=Math.PI){super(),this.type="SphereGeometry",this.parameters={radius:t,widthSegments:e,heightSegments:i,phiStart:s,phiLength:r,thetaStart:a,thetaLength:o},e=Math.max(3,Math.floor(e)),i=Math.max(2,Math.floor(i));let l=Math.min(a+o,Math.PI),c=0,h=[],u=new I,d=new I,f=[],g=[],x=[],m=[];for(let p=0;p<=i;p++){let T=[],M=p/i,y=0;p===0&&a===0?y=.5/e:p===i&&l===Math.PI&&(y=-.5/e);for(let D=0;D<=e;D++){let S=D/e;u.x=-t*Math.cos(s+S*r)*Math.sin(a+M*o),u.y=t*Math.cos(a+M*o),u.z=t*Math.sin(s+S*r)*Math.sin(a+M*o),g.push(u.x,u.y,u.z),d.copy(u).normalize(),x.push(d.x,d.y,d.z),m.push(S+y,1-M),T.push(c++)}h.push(T)}for(let p=0;p<i;p++)for(let T=0;T<e;T++){let M=h[p][T+1],y=h[p][T],D=h[p+1][T],S=h[p+1][T+1];(p!==0||a>0)&&f.push(M,y,S),(p!==i-1||l<Math.PI)&&f.push(y,D,S)}this.setIndex(f),this.setAttribute("position",new _e(g,3)),this.setAttribute("normal",new _e(x,3)),this.setAttribute("uv",new _e(m,2))}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new n(t.radius,t.widthSegments,t.heightSegments,t.phiStart,t.phiLength,t.thetaStart,t.thetaLength)}};var Jo=class extends Te{static get type(){return"RawShaderMaterial"}constructor(t){super(t),this.isRawShaderMaterial=!0}},bn=class extends es{static get type(){return"MeshStandardMaterial"}constructor(t){super(),this.isMeshStandardMaterial=!0,this.defines={STANDARD:""},this.color=new Pt(16777215),this.roughness=1,this.metalness=0,this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.emissive=new Pt(0),this.emissiveIntensity=1,this.emissiveMap=null,this.bumpMap=null,this.bumpScale=1,this.normalMap=null,this.normalMapType=vf,this.normalScale=new j(1,1),this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.roughnessMap=null,this.metalnessMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new ti,this.envMapIntensity=1,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.flatShading=!1,this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.defines={STANDARD:""},this.color.copy(t.color),this.roughness=t.roughness,this.metalness=t.metalness,this.map=t.map,this.lightMap=t.lightMap,this.lightMapIntensity=t.lightMapIntensity,this.aoMap=t.aoMap,this.aoMapIntensity=t.aoMapIntensity,this.emissive.copy(t.emissive),this.emissiveMap=t.emissiveMap,this.emissiveIntensity=t.emissiveIntensity,this.bumpMap=t.bumpMap,this.bumpScale=t.bumpScale,this.normalMap=t.normalMap,this.normalMapType=t.normalMapType,this.normalScale.copy(t.normalScale),this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this.roughnessMap=t.roughnessMap,this.metalnessMap=t.metalnessMap,this.alphaMap=t.alphaMap,this.envMap=t.envMap,this.envMapRotation.copy(t.envMapRotation),this.envMapIntensity=t.envMapIntensity,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.wireframeLinecap=t.wireframeLinecap,this.wireframeLinejoin=t.wireframeLinejoin,this.flatShading=t.flatShading,this.fog=t.fog,this}},Mn=class extends bn{static get type(){return"MeshPhysicalMaterial"}constructor(t){super(),this.isMeshPhysicalMaterial=!0,this.defines={STANDARD:"",PHYSICAL:""},this.anisotropyRotation=0,this.anisotropyMap=null,this.clearcoatMap=null,this.clearcoatRoughness=0,this.clearcoatRoughnessMap=null,this.clearcoatNormalScale=new j(1,1),this.clearcoatNormalMap=null,this.ior=1.5,Object.defineProperty(this,"reflectivity",{get:function(){return qe(2.5*(this.ior-1)/(this.ior+1),0,1)},set:function(e){this.ior=(1+.4*e)/(1-.4*e)}}),this.iridescenceMap=null,this.iridescenceIOR=1.3,this.iridescenceThicknessRange=[100,400],this.iridescenceThicknessMap=null,this.sheenColor=new Pt(0),this.sheenColorMap=null,this.sheenRoughness=1,this.sheenRoughnessMap=null,this.transmissionMap=null,this.thickness=0,this.thicknessMap=null,this.attenuationDistance=1/0,this.attenuationColor=new Pt(1,1,1),this.specularIntensity=1,this.specularIntensityMap=null,this.specularColor=new Pt(1,1,1),this.specularColorMap=null,this._anisotropy=0,this._clearcoat=0,this._dispersion=0,this._iridescence=0,this._sheen=0,this._transmission=0,this.setValues(t)}get anisotropy(){return this._anisotropy}set anisotropy(t){this._anisotropy>0!=t>0&&this.version++,this._anisotropy=t}get clearcoat(){return this._clearcoat}set clearcoat(t){this._clearcoat>0!=t>0&&this.version++,this._clearcoat=t}get iridescence(){return this._iridescence}set iridescence(t){this._iridescence>0!=t>0&&this.version++,this._iridescence=t}get dispersion(){return this._dispersion}set dispersion(t){this._dispersion>0!=t>0&&this.version++,this._dispersion=t}get sheen(){return this._sheen}set sheen(t){this._sheen>0!=t>0&&this.version++,this._sheen=t}get transmission(){return this._transmission}set transmission(t){this._transmission>0!=t>0&&this.version++,this._transmission=t}copy(t){return super.copy(t),this.defines={STANDARD:"",PHYSICAL:""},this.anisotropy=t.anisotropy,this.anisotropyRotation=t.anisotropyRotation,this.anisotropyMap=t.anisotropyMap,this.clearcoat=t.clearcoat,this.clearcoatMap=t.clearcoatMap,this.clearcoatRoughness=t.clearcoatRoughness,this.clearcoatRoughnessMap=t.clearcoatRoughnessMap,this.clearcoatNormalMap=t.clearcoatNormalMap,this.clearcoatNormalScale.copy(t.clearcoatNormalScale),this.dispersion=t.dispersion,this.ior=t.ior,this.iridescence=t.iridescence,this.iridescenceMap=t.iridescenceMap,this.iridescenceIOR=t.iridescenceIOR,this.iridescenceThicknessRange=[...t.iridescenceThicknessRange],this.iridescenceThicknessMap=t.iridescenceThicknessMap,this.sheen=t.sheen,this.sheenColor.copy(t.sheenColor),this.sheenColorMap=t.sheenColorMap,this.sheenRoughness=t.sheenRoughness,this.sheenRoughnessMap=t.sheenRoughnessMap,this.transmission=t.transmission,this.transmissionMap=t.transmissionMap,this.thickness=t.thickness,this.thicknessMap=t.thicknessMap,this.attenuationDistance=t.attenuationDistance,this.attenuationColor.copy(t.attenuationColor),this.specularIntensity=t.specularIntensity,this.specularIntensityMap=t.specularIntensityMap,this.specularColor.copy(t.specularColor),this.specularColorMap=t.specularColorMap,this}};function Eo(n,t,e){return!n||!e&&n.constructor===t?n:typeof t.BYTES_PER_ELEMENT=="number"?new t(n):Array.prototype.slice.call(n)}function Pv(n){return ArrayBuffer.isView(n)&&!(n instanceof DataView)}var kr=class{constructor(t,e,i,s){this.parameterPositions=t,this._cachedIndex=0,this.resultBuffer=s!==void 0?s:new e.constructor(i),this.sampleValues=e,this.valueSize=i,this.settings=null,this.DefaultSettings_={}}evaluate(t){let e=this.parameterPositions,i=this._cachedIndex,s=e[i],r=e[i-1];n:{t:{let a;e:{i:if(!(t<s)){for(let o=i+2;;){if(s===void 0){if(t<r)break i;return i=e.length,this._cachedIndex=i,this.copySampleValue_(i-1)}if(i===o)break;if(r=s,s=e[++i],t<s)break t}a=e.length;break e}if(!(t>=r)){let o=e[1];t<o&&(i=2,r=o);for(let l=i-2;;){if(r===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(i===l)break;if(s=r,r=e[--i-1],t>=r)break t}a=i,i=0;break e}break n}for(;i<a;){let o=i+a>>>1;t<e[o]?a=o:i=o+1}if(s=e[i],r=e[i-1],r===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(s===void 0)return i=e.length,this._cachedIndex=i,this.copySampleValue_(i-1)}this._cachedIndex=i,this.intervalChanged_(i,r,s)}return this.interpolate_(i,r,t,s)}getSettings_(){return this.settings||this.DefaultSettings_}copySampleValue_(t){let e=this.resultBuffer,i=this.sampleValues,s=this.valueSize,r=t*s;for(let a=0;a!==s;++a)e[a]=i[r+a];return e}interpolate_(){throw new Error("call to abstract method")}intervalChanged_(){}},Uh=class extends kr{constructor(t,e,i,s){super(t,e,i,s),this._weightPrev=-0,this._offsetPrev=-0,this._weightNext=-0,this._offsetNext=-0,this.DefaultSettings_={endingStart:Ku,endingEnd:Ku}}intervalChanged_(t,e,i){let s=this.parameterPositions,r=t-2,a=t+1,o=s[r],l=s[a];if(o===void 0)switch(this.getSettings_().endingStart){case Qu:r=t,o=2*e-i;break;case ju:r=s.length-2,o=e+s[r]-s[r+1];break;default:r=t,o=i}if(l===void 0)switch(this.getSettings_().endingEnd){case Qu:a=t,l=2*i-e;break;case ju:a=1,l=i+s[1]-s[0];break;default:a=t-1,l=e}let c=(i-e)*.5,h=this.valueSize;this._weightPrev=c/(e-o),this._weightNext=c/(l-i),this._offsetPrev=r*h,this._offsetNext=a*h}interpolate_(t,e,i,s){let r=this.resultBuffer,a=this.sampleValues,o=this.valueSize,l=t*o,c=l-o,h=this._offsetPrev,u=this._offsetNext,d=this._weightPrev,f=this._weightNext,g=(i-e)/(s-e),x=g*g,m=x*g,p=-d*m+2*d*x-d*g,T=(1+d)*m+(-1.5-2*d)*x+(-.5+d)*g+1,M=(-1-f)*m+(1.5+f)*x+.5*g,y=f*m-f*x;for(let D=0;D!==o;++D)r[D]=p*a[h+D]+T*a[c+D]+M*a[l+D]+y*a[u+D];return r}},Nh=class extends kr{constructor(t,e,i,s){super(t,e,i,s)}interpolate_(t,e,i,s){let r=this.resultBuffer,a=this.sampleValues,o=this.valueSize,l=t*o,c=l-o,h=(i-e)/(s-e),u=1-h;for(let d=0;d!==o;++d)r[d]=a[c+d]*u+a[l+d]*h;return r}},Bh=class extends kr{constructor(t,e,i,s){super(t,e,i,s)}interpolate_(t){return this.copySampleValue_(t-1)}},Gn=class{constructor(t,e,i,s){if(t===void 0)throw new Error("THREE.KeyframeTrack: track name is undefined");if(e===void 0||e.length===0)throw new Error("THREE.KeyframeTrack: no keyframes in track named "+t);this.name=t,this.times=Eo(e,this.TimeBufferType),this.values=Eo(i,this.ValueBufferType),this.setInterpolation(s||this.DefaultInterpolation)}static toJSON(t){let e=t.constructor,i;if(e.toJSON!==this.toJSON)i=e.toJSON(t);else{i={name:t.name,times:Eo(t.times,Array),values:Eo(t.values,Array)};let s=t.getInterpolation();s!==t.DefaultInterpolation&&(i.interpolation=s)}return i.type=t.ValueTypeName,i}InterpolantFactoryMethodDiscrete(t){return new Bh(this.times,this.values,this.getValueSize(),t)}InterpolantFactoryMethodLinear(t){return new Nh(this.times,this.values,this.getValueSize(),t)}InterpolantFactoryMethodSmooth(t){return new Uh(this.times,this.values,this.getValueSize(),t)}setInterpolation(t){let e;switch(t){case Io:e=this.InterpolantFactoryMethodDiscrete;break;case rh:e=this.InterpolantFactoryMethodLinear;break;case zl:e=this.InterpolantFactoryMethodSmooth;break}if(e===void 0){let i="unsupported interpolation for "+this.ValueTypeName+" keyframe track named "+this.name;if(this.createInterpolant===void 0)if(t!==this.DefaultInterpolation)this.setInterpolation(this.DefaultInterpolation);else throw new Error(i);return console.warn("THREE.KeyframeTrack:",i),this}return this.createInterpolant=e,this}getInterpolation(){switch(this.createInterpolant){case this.InterpolantFactoryMethodDiscrete:return Io;case this.InterpolantFactoryMethodLinear:return rh;case this.InterpolantFactoryMethodSmooth:return zl}}getValueSize(){return this.values.length/this.times.length}shift(t){if(t!==0){let e=this.times;for(let i=0,s=e.length;i!==s;++i)e[i]+=t}return this}scale(t){if(t!==1){let e=this.times;for(let i=0,s=e.length;i!==s;++i)e[i]*=t}return this}trim(t,e){let i=this.times,s=i.length,r=0,a=s-1;for(;r!==s&&i[r]<t;)++r;for(;a!==-1&&i[a]>e;)--a;if(++a,r!==0||a!==s){r>=a&&(a=Math.max(a,1),r=a-1);let o=this.getValueSize();this.times=i.slice(r,a),this.values=this.values.slice(r*o,a*o)}return this}validate(){let t=!0,e=this.getValueSize();e-Math.floor(e)!==0&&(console.error("THREE.KeyframeTrack: Invalid value size in track.",this),t=!1);let i=this.times,s=this.values,r=i.length;r===0&&(console.error("THREE.KeyframeTrack: Track is empty.",this),t=!1);let a=null;for(let o=0;o!==r;o++){let l=i[o];if(typeof l=="number"&&isNaN(l)){console.error("THREE.KeyframeTrack: Time is not a valid number.",this,o,l),t=!1;break}if(a!==null&&a>l){console.error("THREE.KeyframeTrack: Out of order keys.",this,o,l,a),t=!1;break}a=l}if(s!==void 0&&Pv(s))for(let o=0,l=s.length;o!==l;++o){let c=s[o];if(isNaN(c)){console.error("THREE.KeyframeTrack: Value is not a valid number.",this,o,c),t=!1;break}}return t}optimize(){let t=this.times.slice(),e=this.values.slice(),i=this.getValueSize(),s=this.getInterpolation()===zl,r=t.length-1,a=1;for(let o=1;o<r;++o){let l=!1,c=t[o],h=t[o+1];if(c!==h&&(o!==1||c!==t[0]))if(s)l=!0;else{let u=o*i,d=u-i,f=u+i;for(let g=0;g!==i;++g){let x=e[u+g];if(x!==e[d+g]||x!==e[f+g]){l=!0;break}}}if(l){if(o!==a){t[a]=t[o];let u=o*i,d=a*i;for(let f=0;f!==i;++f)e[d+f]=e[u+f]}++a}}if(r>0){t[a]=t[r];for(let o=r*i,l=a*i,c=0;c!==i;++c)e[l+c]=e[o+c];++a}return a!==t.length?(this.times=t.slice(0,a),this.values=e.slice(0,a*i)):(this.times=t,this.values=e),this}clone(){let t=this.times.slice(),e=this.values.slice(),i=this.constructor,s=new i(this.name,t,e);return s.createInterpolant=this.createInterpolant,s}};Gn.prototype.TimeBufferType=Float32Array;Gn.prototype.ValueBufferType=Float32Array;Gn.prototype.DefaultInterpolation=rh;var Os=class extends Gn{constructor(t,e,i){super(t,e,i)}};Os.prototype.ValueTypeName="bool";Os.prototype.ValueBufferType=Array;Os.prototype.DefaultInterpolation=Io;Os.prototype.InterpolantFactoryMethodLinear=void 0;Os.prototype.InterpolantFactoryMethodSmooth=void 0;var Oh=class extends Gn{};Oh.prototype.ValueTypeName="color";var zh=class extends Gn{};zh.prototype.ValueTypeName="number";var Hh=class extends kr{constructor(t,e,i,s){super(t,e,i,s)}interpolate_(t,e,i,s){let r=this.resultBuffer,a=this.sampleValues,o=this.valueSize,l=(i-e)/(s-e),c=t*o;for(let h=c+o;c!==h;c+=4)ts.slerpFlat(r,0,a,c-o,a,c,l);return r}},Ko=class extends Gn{InterpolantFactoryMethodLinear(t){return new Hh(this.times,this.values,this.getValueSize(),t)}};Ko.prototype.ValueTypeName="quaternion";Ko.prototype.InterpolantFactoryMethodSmooth=void 0;var zs=class extends Gn{constructor(t,e,i){super(t,e,i)}};zs.prototype.ValueTypeName="string";zs.prototype.ValueBufferType=Array;zs.prototype.DefaultInterpolation=Io;zs.prototype.InterpolantFactoryMethodLinear=void 0;zs.prototype.InterpolantFactoryMethodSmooth=void 0;var Vh=class extends Gn{};Vh.prototype.ValueTypeName="vector";var tf={enabled:!1,files:{},add:function(n,t){this.enabled!==!1&&(this.files[n]=t)},get:function(n){if(this.enabled!==!1)return this.files[n]},remove:function(n){delete this.files[n]},clear:function(){this.files={}}},Gh=class{constructor(t,e,i){let s=this,r=!1,a=0,o=0,l,c=[];this.onStart=void 0,this.onLoad=t,this.onProgress=e,this.onError=i,this.itemStart=function(h){o++,r===!1&&s.onStart!==void 0&&s.onStart(h,a,o),r=!0},this.itemEnd=function(h){a++,s.onProgress!==void 0&&s.onProgress(h,a,o),a===o&&(r=!1,s.onLoad!==void 0&&s.onLoad())},this.itemError=function(h){s.onError!==void 0&&s.onError(h)},this.resolveURL=function(h){return l?l(h):h},this.setURLModifier=function(h){return l=h,this},this.addHandler=function(h,u){return c.push(h,u),this},this.removeHandler=function(h){let u=c.indexOf(h);return u!==-1&&c.splice(u,2),this},this.getHandler=function(h){for(let u=0,d=c.length;u<d;u+=2){let f=c[u],g=c[u+1];if(f.global&&(f.lastIndex=0),f.test(h))return g}return null}}},Iv=new Gh,Sa=class{constructor(t){this.manager=t!==void 0?t:Iv,this.crossOrigin="anonymous",this.withCredentials=!1,this.path="",this.resourcePath="",this.requestHeader={}}load(){}loadAsync(t,e){let i=this;return new Promise(function(s,r){i.load(t,s,e,r)})}parse(){}setCrossOrigin(t){return this.crossOrigin=t,this}setWithCredentials(t){return this.withCredentials=t,this}setPath(t){return this.path=t,this}setResourcePath(t){return this.resourcePath=t,this}setRequestHeader(t){return this.requestHeader=t,this}};Sa.DEFAULT_MATERIAL_NAME="__DEFAULT";var Wh=class extends Sa{constructor(t){super(t)}load(t,e,i,s){this.path!==void 0&&(t=this.path+t),t=this.manager.resolveURL(t);let r=this,a=tf.get(t);if(a!==void 0)return r.manager.itemStart(t),setTimeout(function(){e&&e(a),r.manager.itemEnd(t)},0),a;let o=fa("img");function l(){h(),tf.add(t,this),e&&e(this),r.manager.itemEnd(t)}function c(u){h(),s&&s(u),r.manager.itemError(t),r.manager.itemEnd(t)}function h(){o.removeEventListener("load",l,!1),o.removeEventListener("error",c,!1)}return o.addEventListener("load",l,!1),o.addEventListener("error",c,!1),t.slice(0,5)!=="data:"&&this.crossOrigin!==void 0&&(o.crossOrigin=this.crossOrigin),r.manager.itemStart(t),o.src=t,o}};var wa=class extends Sa{constructor(t){super(t)}load(t,e,i,s){let r=new on,a=new Wh(this.manager);return a.setCrossOrigin(this.crossOrigin),a.setPath(this.path),a.load(t,function(o){r.image=o,r.needsUpdate=!0,e!==void 0&&e(r)},i,s),r}},Ea=class extends ln{constructor(t,e=1){super(),this.isLight=!0,this.type="Light",this.color=new Pt(t),this.intensity=e}dispose(){}copy(t,e){return super.copy(t,e),this.color.copy(t.color),this.intensity=t.intensity,this}toJSON(t){let e=super.toJSON(t);return e.object.color=this.color.getHex(),e.object.intensity=this.intensity,this.groundColor!==void 0&&(e.object.groundColor=this.groundColor.getHex()),this.distance!==void 0&&(e.object.distance=this.distance),this.angle!==void 0&&(e.object.angle=this.angle),this.decay!==void 0&&(e.object.decay=this.decay),this.penumbra!==void 0&&(e.object.penumbra=this.penumbra),this.shadow!==void 0&&(e.object.shadow=this.shadow.toJSON()),this.target!==void 0&&(e.object.target=this.target.uuid),e}};var yc=new pe,ef=new I,nf=new I,Qo=class{constructor(t){this.camera=t,this.intensity=1,this.bias=0,this.normalBias=0,this.radius=1,this.blurSamples=8,this.mapSize=new j(512,512),this.map=null,this.mapPass=null,this.matrix=new pe,this.autoUpdate=!0,this.needsUpdate=!1,this._frustum=new pa,this._frameExtents=new j(1,1),this._viewportCount=1,this._viewports=[new xe(0,0,1,1)]}getViewportCount(){return this._viewportCount}getFrustum(){return this._frustum}updateMatrices(t){let e=this.camera,i=this.matrix;ef.setFromMatrixPosition(t.matrixWorld),e.position.copy(ef),nf.setFromMatrixPosition(t.target.matrixWorld),e.lookAt(nf),e.updateMatrixWorld(),yc.multiplyMatrices(e.projectionMatrix,e.matrixWorldInverse),this._frustum.setFromProjectionMatrix(yc),i.set(.5,0,0,.5,0,.5,0,.5,0,0,.5,.5,0,0,0,1),i.multiply(yc)}getViewport(t){return this._viewports[t]}getFrameExtents(){return this._frameExtents}dispose(){this.map&&this.map.dispose(),this.mapPass&&this.mapPass.dispose()}copy(t){return this.camera=t.camera.clone(),this.intensity=t.intensity,this.bias=t.bias,this.radius=t.radius,this.mapSize.copy(t.mapSize),this}clone(){return new this.constructor().copy(this)}toJSON(){let t={};return this.intensity!==1&&(t.intensity=this.intensity),this.bias!==0&&(t.bias=this.bias),this.normalBias!==0&&(t.normalBias=this.normalBias),this.radius!==1&&(t.radius=this.radius),(this.mapSize.x!==512||this.mapSize.y!==512)&&(t.mapSize=this.mapSize.toArray()),t.camera=this.camera.toJSON(!1).object,delete t.camera.matrix,t}};var sf=new pe,ra=new I,_c=new I,qh=class extends Qo{constructor(){super(new Qe(90,1,.5,500)),this.isPointLightShadow=!0,this._frameExtents=new j(4,2),this._viewportCount=6,this._viewports=[new xe(2,1,1,1),new xe(0,1,1,1),new xe(3,1,1,1),new xe(1,1,1,1),new xe(3,0,1,1),new xe(1,0,1,1)],this._cubeDirections=[new I(1,0,0),new I(-1,0,0),new I(0,0,1),new I(0,0,-1),new I(0,1,0),new I(0,-1,0)],this._cubeUps=[new I(0,1,0),new I(0,1,0),new I(0,1,0),new I(0,1,0),new I(0,0,1),new I(0,0,-1)]}updateMatrices(t,e=0){let i=this.camera,s=this.matrix,r=t.distance||i.far;r!==i.far&&(i.far=r,i.updateProjectionMatrix()),ra.setFromMatrixPosition(t.matrixWorld),i.position.copy(ra),_c.copy(i.position),_c.add(this._cubeDirections[e]),i.up.copy(this._cubeUps[e]),i.lookAt(_c),i.updateMatrixWorld(),s.makeTranslation(-ra.x,-ra.y,-ra.z),sf.multiplyMatrices(i.projectionMatrix,i.matrixWorldInverse),this._frustum.setFromProjectionMatrix(sf)}},jo=class extends Ea{constructor(t,e,i=0,s=2){super(t,e),this.isPointLight=!0,this.type="PointLight",this.distance=i,this.decay=s,this.shadow=new qh}get power(){return this.intensity*4*Math.PI}set power(t){this.intensity=t/(4*Math.PI)}dispose(){this.shadow.dispose()}copy(t,e){return super.copy(t,e),this.distance=t.distance,this.decay=t.decay,this.shadow=t.shadow.clone(),this}},Xh=class extends Qo{constructor(){super(new Ir(-5,5,5,-5,.5,500)),this.isDirectionalLightShadow=!0}},tl=class extends Ea{constructor(t,e){super(t,e),this.isDirectionalLight=!0,this.type="DirectionalLight",this.position.copy(ln.DEFAULT_UP),this.updateMatrix(),this.target=new ln,this.shadow=new Xh}dispose(){this.shadow.dispose()}copy(t){return super.copy(t),this.target=t.target.clone(),this.shadow=t.shadow.clone(),this}},el=class extends Ea{constructor(t,e){super(t,e),this.isAmbientLight=!0,this.type="AmbientLight"}};var nl=class{constructor(t=!0){this.autoStart=t,this.startTime=0,this.oldTime=0,this.elapsedTime=0,this.running=!1}start(){this.startTime=rf(),this.oldTime=this.startTime,this.elapsedTime=0,this.running=!0}stop(){this.getElapsedTime(),this.running=!1,this.autoStart=!1}getElapsedTime(){return this.getDelta(),this.elapsedTime}getDelta(){let t=0;if(this.autoStart&&!this.running)return this.start(),0;if(this.running){let e=rf();t=(e-this.oldTime)/1e3,this.oldTime=e,this.elapsedTime+=t}return t}};function rf(){return performance.now()}var hu="\\[\\]\\.:\\/",Fv=new RegExp("["+hu+"]","g"),uu="[^"+hu+"]",Dv="[^"+hu.replace("\\.","")+"]",kv=/((?:WC+[\/:])*)/.source.replace("WC",uu),Lv=/(WCOD+)?/.source.replace("WCOD",Dv),Uv=/(?:\.(WC+)(?:\[(.+)\])?)?/.source.replace("WC",uu),Nv=/\.(WC+)(?:\[(.+)\])?/.source.replace("WC",uu),Bv=new RegExp("^"+kv+Lv+Uv+Nv+"$"),Ov=["material","materials","bones","map"],$h=class{constructor(t,e,i){let s=i||Ee.parseTrackName(e);this._targetGroup=t,this._bindings=t.subscribe_(e,s)}getValue(t,e){this.bind();let i=this._targetGroup.nCachedObjects_,s=this._bindings[i];s!==void 0&&s.getValue(t,e)}setValue(t,e){let i=this._bindings;for(let s=this._targetGroup.nCachedObjects_,r=i.length;s!==r;++s)i[s].setValue(t,e)}bind(){let t=this._bindings;for(let e=this._targetGroup.nCachedObjects_,i=t.length;e!==i;++e)t[e].bind()}unbind(){let t=this._bindings;for(let e=this._targetGroup.nCachedObjects_,i=t.length;e!==i;++e)t[e].unbind()}},Ee=class n{constructor(t,e,i){this.path=e,this.parsedPath=i||n.parseTrackName(e),this.node=n.findNode(t,this.parsedPath.nodeName),this.rootNode=t,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}static create(t,e,i){return t&&t.isAnimationObjectGroup?new n.Composite(t,e,i):new n(t,e,i)}static sanitizeNodeName(t){return t.replace(/\s/g,"_").replace(Fv,"")}static parseTrackName(t){let e=Bv.exec(t);if(e===null)throw new Error("PropertyBinding: Cannot parse trackName: "+t);let i={nodeName:e[2],objectName:e[3],objectIndex:e[4],propertyName:e[5],propertyIndex:e[6]},s=i.nodeName&&i.nodeName.lastIndexOf(".");if(s!==void 0&&s!==-1){let r=i.nodeName.substring(s+1);Ov.indexOf(r)!==-1&&(i.nodeName=i.nodeName.substring(0,s),i.objectName=r)}if(i.propertyName===null||i.propertyName.length===0)throw new Error("PropertyBinding: can not parse propertyName from trackName: "+t);return i}static findNode(t,e){if(e===void 0||e===""||e==="."||e===-1||e===t.name||e===t.uuid)return t;if(t.skeleton){let i=t.skeleton.getBoneByName(e);if(i!==void 0)return i}if(t.children){let i=function(r){for(let a=0;a<r.length;a++){let o=r[a];if(o.name===e||o.uuid===e)return o;let l=i(o.children);if(l)return l}return null},s=i(t.children);if(s)return s}return null}_getValue_unavailable(){}_setValue_unavailable(){}_getValue_direct(t,e){t[e]=this.targetObject[this.propertyName]}_getValue_array(t,e){let i=this.resolvedProperty;for(let s=0,r=i.length;s!==r;++s)t[e++]=i[s]}_getValue_arrayElement(t,e){t[e]=this.resolvedProperty[this.propertyIndex]}_getValue_toArray(t,e){this.resolvedProperty.toArray(t,e)}_setValue_direct(t,e){this.targetObject[this.propertyName]=t[e]}_setValue_direct_setNeedsUpdate(t,e){this.targetObject[this.propertyName]=t[e],this.targetObject.needsUpdate=!0}_setValue_direct_setMatrixWorldNeedsUpdate(t,e){this.targetObject[this.propertyName]=t[e],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_array(t,e){let i=this.resolvedProperty;for(let s=0,r=i.length;s!==r;++s)i[s]=t[e++]}_setValue_array_setNeedsUpdate(t,e){let i=this.resolvedProperty;for(let s=0,r=i.length;s!==r;++s)i[s]=t[e++];this.targetObject.needsUpdate=!0}_setValue_array_setMatrixWorldNeedsUpdate(t,e){let i=this.resolvedProperty;for(let s=0,r=i.length;s!==r;++s)i[s]=t[e++];this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_arrayElement(t,e){this.resolvedProperty[this.propertyIndex]=t[e]}_setValue_arrayElement_setNeedsUpdate(t,e){this.resolvedProperty[this.propertyIndex]=t[e],this.targetObject.needsUpdate=!0}_setValue_arrayElement_setMatrixWorldNeedsUpdate(t,e){this.resolvedProperty[this.propertyIndex]=t[e],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_fromArray(t,e){this.resolvedProperty.fromArray(t,e)}_setValue_fromArray_setNeedsUpdate(t,e){this.resolvedProperty.fromArray(t,e),this.targetObject.needsUpdate=!0}_setValue_fromArray_setMatrixWorldNeedsUpdate(t,e){this.resolvedProperty.fromArray(t,e),this.targetObject.matrixWorldNeedsUpdate=!0}_getValue_unbound(t,e){this.bind(),this.getValue(t,e)}_setValue_unbound(t,e){this.bind(),this.setValue(t,e)}bind(){let t=this.node,e=this.parsedPath,i=e.objectName,s=e.propertyName,r=e.propertyIndex;if(t||(t=n.findNode(this.rootNode,e.nodeName),this.node=t),this.getValue=this._getValue_unavailable,this.setValue=this._setValue_unavailable,!t){console.warn("THREE.PropertyBinding: No target node found for track: "+this.path+".");return}if(i){let c=e.objectIndex;switch(i){case"materials":if(!t.material){console.error("THREE.PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!t.material.materials){console.error("THREE.PropertyBinding: Can not bind to material.materials as node.material does not have a materials array.",this);return}t=t.material.materials;break;case"bones":if(!t.skeleton){console.error("THREE.PropertyBinding: Can not bind to bones as node does not have a skeleton.",this);return}t=t.skeleton.bones;for(let h=0;h<t.length;h++)if(t[h].name===c){c=h;break}break;case"map":if("map"in t){t=t.map;break}if(!t.material){console.error("THREE.PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!t.material.map){console.error("THREE.PropertyBinding: Can not bind to material.map as node.material does not have a map.",this);return}t=t.material.map;break;default:if(t[i]===void 0){console.error("THREE.PropertyBinding: Can not bind to objectName of node undefined.",this);return}t=t[i]}if(c!==void 0){if(t[c]===void 0){console.error("THREE.PropertyBinding: Trying to bind to objectIndex of objectName, but is undefined.",this,t);return}t=t[c]}}let a=t[s];if(a===void 0){let c=e.nodeName;console.error("THREE.PropertyBinding: Trying to update property for track: "+c+"."+s+" but it wasn't found.",t);return}let o=this.Versioning.None;this.targetObject=t,t.needsUpdate!==void 0?o=this.Versioning.NeedsUpdate:t.matrixWorldNeedsUpdate!==void 0&&(o=this.Versioning.MatrixWorldNeedsUpdate);let l=this.BindingType.Direct;if(r!==void 0){if(s==="morphTargetInfluences"){if(!t.geometry){console.error("THREE.PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.",this);return}if(!t.geometry.morphAttributes){console.error("THREE.PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.morphAttributes.",this);return}t.morphTargetDictionary[r]!==void 0&&(r=t.morphTargetDictionary[r])}l=this.BindingType.ArrayElement,this.resolvedProperty=a,this.propertyIndex=r}else a.fromArray!==void 0&&a.toArray!==void 0?(l=this.BindingType.HasFromToArray,this.resolvedProperty=a):Array.isArray(a)?(l=this.BindingType.EntireArray,this.resolvedProperty=a):this.propertyName=s;this.getValue=this.GetterByBindingType[l],this.setValue=this.SetterByBindingTypeAndVersioning[l][o]}unbind(){this.node=null,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}};Ee.Composite=$h;Ee.prototype.BindingType={Direct:0,EntireArray:1,ArrayElement:2,HasFromToArray:3};Ee.prototype.Versioning={None:0,NeedsUpdate:1,MatrixWorldNeedsUpdate:2};Ee.prototype.GetterByBindingType=[Ee.prototype._getValue_direct,Ee.prototype._getValue_array,Ee.prototype._getValue_arrayElement,Ee.prototype._getValue_toArray];Ee.prototype.SetterByBindingTypeAndVersioning=[[Ee.prototype._setValue_direct,Ee.prototype._setValue_direct_setNeedsUpdate,Ee.prototype._setValue_direct_setMatrixWorldNeedsUpdate],[Ee.prototype._setValue_array,Ee.prototype._setValue_array_setNeedsUpdate,Ee.prototype._setValue_array_setMatrixWorldNeedsUpdate],[Ee.prototype._setValue_arrayElement,Ee.prototype._setValue_arrayElement_setNeedsUpdate,Ee.prototype._setValue_arrayElement_setMatrixWorldNeedsUpdate],[Ee.prototype._setValue_fromArray,Ee.prototype._setValue_fromArray_setNeedsUpdate,Ee.prototype._setValue_fromArray_setMatrixWorldNeedsUpdate]];var f_=new Float32Array(1);typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("register",{detail:{revision:"170"}}));typeof window<"u"&&(window.__THREE__?console.warn("WARNING: Multiple instances of Three.js being imported."):window.__THREE__="170");var Nr={name:"CopyShader",uniforms:{tDiffuse:{value:null},opacity:{value:1}},vertexShader:`

		varying vec2 vUv;

		void main() {

			vUv = uv;
			gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );

		}`,fragmentShader:`

		uniform float opacity;

		uniform sampler2D tDiffuse;

		varying vec2 vUv;

		void main() {

			vec4 texel = texture2D( tDiffuse, vUv );
			gl_FragColor = opacity * texel;


		}`};var cn=class{constructor(){this.isPass=!0,this.enabled=!0,this.needsSwap=!0,this.clear=!1,this.renderToScreen=!1}setSize(){}render(){console.error("THREE.Pass: .render() must be implemented in derived pass.")}dispose(){}},Hv=new Ir(-1,1,1,-1,0,1),du=class extends tn{constructor(){super(),this.setAttribute("position",new _e([-1,3,0,-1,-1,0,3,-1,0],3)),this.setAttribute("uv",new _e([0,2,0,0,2,0],2))}},Vv=new du,si=class{constructor(t){this._mesh=new Ot(Vv,t)}dispose(){this._mesh.geometry.dispose()}render(t){t.render(this._mesh,Hv)}get material(){return this._mesh.material}set material(t){this._mesh.material=t}};var Hs=class extends cn{constructor(t,e){super(),this.textureID=e!==void 0?e:"tDiffuse",t instanceof Te?(this.uniforms=t.uniforms,this.material=t):t&&(this.uniforms=ii.clone(t.uniforms),this.material=new Te({name:t.name!==void 0?t.name:"unspecified",defines:Object.assign({},t.defines),uniforms:this.uniforms,vertexShader:t.vertexShader,fragmentShader:t.fragmentShader})),this.fsQuad=new si(this.material)}render(t,e,i){this.uniforms[this.textureID]&&(this.uniforms[this.textureID].value=i.texture),this.fsQuad.material=this.material,this.renderToScreen?(t.setRenderTarget(null),this.fsQuad.render(t)):(t.setRenderTarget(e),this.clear&&t.clear(t.autoClearColor,t.autoClearDepth,t.autoClearStencil),this.fsQuad.render(t))}dispose(){this.material.dispose(),this.fsQuad.dispose()}};var Aa=class extends cn{constructor(t,e){super(),this.scene=t,this.camera=e,this.clear=!0,this.needsSwap=!1,this.inverse=!1}render(t,e,i){let s=t.getContext(),r=t.state;r.buffers.color.setMask(!1),r.buffers.depth.setMask(!1),r.buffers.color.setLocked(!0),r.buffers.depth.setLocked(!0);let a,o;this.inverse?(a=0,o=1):(a=1,o=0),r.buffers.stencil.setTest(!0),r.buffers.stencil.setOp(s.REPLACE,s.REPLACE,s.REPLACE),r.buffers.stencil.setFunc(s.ALWAYS,a,4294967295),r.buffers.stencil.setClear(o),r.buffers.stencil.setLocked(!0),t.setRenderTarget(i),this.clear&&t.clear(),t.render(this.scene,this.camera),t.setRenderTarget(e),this.clear&&t.clear(),t.render(this.scene,this.camera),r.buffers.color.setLocked(!1),r.buffers.depth.setLocked(!1),r.buffers.color.setMask(!0),r.buffers.depth.setMask(!0),r.buffers.stencil.setLocked(!1),r.buffers.stencil.setFunc(s.EQUAL,1,4294967295),r.buffers.stencil.setOp(s.KEEP,s.KEEP,s.KEEP),r.buffers.stencil.setLocked(!0)}},ll=class extends cn{constructor(){super(),this.needsSwap=!1}render(t){t.state.buffers.stencil.setLocked(!1),t.state.buffers.stencil.setTest(!1)}};var cl=class{constructor(t,e){if(this.renderer=t,this._pixelRatio=t.getPixelRatio(),e===void 0){let i=t.getSize(new j);this._width=i.width,this._height=i.height,e=new Ue(this._width*this._pixelRatio,this._height*this._pixelRatio,{type:en}),e.texture.name="EffectComposer.rt1"}else this._width=e.width,this._height=e.height;this.renderTarget1=e,this.renderTarget2=e.clone(),this.renderTarget2.texture.name="EffectComposer.rt2",this.writeBuffer=this.renderTarget1,this.readBuffer=this.renderTarget2,this.renderToScreen=!0,this.passes=[],this.copyPass=new Hs(Nr),this.copyPass.material.blending=Rn,this.clock=new nl}swapBuffers(){let t=this.readBuffer;this.readBuffer=this.writeBuffer,this.writeBuffer=t}addPass(t){this.passes.push(t),t.setSize(this._width*this._pixelRatio,this._height*this._pixelRatio)}insertPass(t,e){this.passes.splice(e,0,t),t.setSize(this._width*this._pixelRatio,this._height*this._pixelRatio)}removePass(t){let e=this.passes.indexOf(t);e!==-1&&this.passes.splice(e,1)}isLastEnabledPass(t){for(let e=t+1;e<this.passes.length;e++)if(this.passes[e].enabled)return!1;return!0}render(t){t===void 0&&(t=this.clock.getDelta());let e=this.renderer.getRenderTarget(),i=!1;for(let s=0,r=this.passes.length;s<r;s++){let a=this.passes[s];if(a.enabled!==!1){if(a.renderToScreen=this.renderToScreen&&this.isLastEnabledPass(s),a.render(this.renderer,this.writeBuffer,this.readBuffer,t,i),a.needsSwap){if(i){let o=this.renderer.getContext(),l=this.renderer.state.buffers.stencil;l.setFunc(o.NOTEQUAL,1,4294967295),this.copyPass.render(this.renderer,this.writeBuffer,this.readBuffer,t),l.setFunc(o.EQUAL,1,4294967295)}this.swapBuffers()}Aa!==void 0&&(a instanceof Aa?i=!0:a instanceof ll&&(i=!1))}}this.renderer.setRenderTarget(e)}reset(t){if(t===void 0){let e=this.renderer.getSize(new j);this._pixelRatio=this.renderer.getPixelRatio(),this._width=e.width,this._height=e.height,t=this.renderTarget1.clone(),t.setSize(this._width*this._pixelRatio,this._height*this._pixelRatio)}this.renderTarget1.dispose(),this.renderTarget2.dispose(),this.renderTarget1=t,this.renderTarget2=t.clone(),this.writeBuffer=this.renderTarget1,this.readBuffer=this.renderTarget2}setSize(t,e){this._width=t,this._height=e;let i=this._width*this._pixelRatio,s=this._height*this._pixelRatio;this.renderTarget1.setSize(i,s),this.renderTarget2.setSize(i,s);for(let r=0;r<this.passes.length;r++)this.passes[r].setSize(i,s)}setPixelRatio(t){this._pixelRatio=t,this.setSize(this._width,this._height)}dispose(){this.renderTarget1.dispose(),this.renderTarget2.dispose(),this.copyPass.dispose()}};var hl=class extends cn{constructor(t,e,i=null,s=null,r=null){super(),this.scene=t,this.camera=e,this.overrideMaterial=i,this.clearColor=s,this.clearAlpha=r,this.clear=!0,this.clearDepth=!1,this.needsSwap=!1,this._oldClearColor=new Pt}render(t,e,i){let s=t.autoClear;t.autoClear=!1;let r,a;this.overrideMaterial!==null&&(a=this.scene.overrideMaterial,this.scene.overrideMaterial=this.overrideMaterial),this.clearColor!==null&&(t.getClearColor(this._oldClearColor),t.setClearColor(this.clearColor,t.getClearAlpha())),this.clearAlpha!==null&&(r=t.getClearAlpha(),t.setClearAlpha(this.clearAlpha)),this.clearDepth==!0&&t.clearDepth(),t.setRenderTarget(this.renderToScreen?null:i),this.clear===!0&&t.clear(t.autoClearColor,t.autoClearDepth,t.autoClearStencil),t.render(this.scene,this.camera),this.clearColor!==null&&t.setClearColor(this._oldClearColor),this.clearAlpha!==null&&t.setClearAlpha(r),this.overrideMaterial!==null&&(this.scene.overrideMaterial=a),t.autoClear=s}};var Ff={name:"LuminosityHighPassShader",shaderID:"luminosityHighPass",uniforms:{tDiffuse:{value:null},luminosityThreshold:{value:1},smoothWidth:{value:1},defaultColor:{value:new Pt(0)},defaultOpacity:{value:0}},vertexShader:`

		varying vec2 vUv;

		void main() {

			vUv = uv;

			gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );

		}`,fragmentShader:`

		uniform sampler2D tDiffuse;
		uniform vec3 defaultColor;
		uniform float defaultOpacity;
		uniform float luminosityThreshold;
		uniform float smoothWidth;

		varying vec2 vUv;

		void main() {

			vec4 texel = texture2D( tDiffuse, vUv );

			float v = luminance( texel.xyz );

			vec4 outputColor = vec4( defaultColor.rgb, defaultOpacity );

			float alpha = smoothstep( luminosityThreshold, luminosityThreshold + smoothWidth, v );

			gl_FragColor = mix( outputColor, texel, alpha );

		}`};var Br=class n extends cn{constructor(t,e,i,s){super(),this.strength=e!==void 0?e:1,this.radius=i,this.threshold=s,this.resolution=t!==void 0?new j(t.x,t.y):new j(256,256),this.clearColor=new Pt(0,0,0),this.renderTargetsHorizontal=[],this.renderTargetsVertical=[],this.nMips=5;let r=Math.round(this.resolution.x/2),a=Math.round(this.resolution.y/2);this.renderTargetBright=new Ue(r,a,{type:en}),this.renderTargetBright.texture.name="UnrealBloomPass.bright",this.renderTargetBright.texture.generateMipmaps=!1;for(let u=0;u<this.nMips;u++){let d=new Ue(r,a,{type:en});d.texture.name="UnrealBloomPass.h"+u,d.texture.generateMipmaps=!1,this.renderTargetsHorizontal.push(d);let f=new Ue(r,a,{type:en});f.texture.name="UnrealBloomPass.v"+u,f.texture.generateMipmaps=!1,this.renderTargetsVertical.push(f),r=Math.round(r/2),a=Math.round(a/2)}let o=Ff;this.highPassUniforms=ii.clone(o.uniforms),this.highPassUniforms.luminosityThreshold.value=s,this.highPassUniforms.smoothWidth.value=.01,this.materialHighPassFilter=new Te({uniforms:this.highPassUniforms,vertexShader:o.vertexShader,fragmentShader:o.fragmentShader}),this.separableBlurMaterials=[];let l=[3,5,7,9,11];r=Math.round(this.resolution.x/2),a=Math.round(this.resolution.y/2);for(let u=0;u<this.nMips;u++)this.separableBlurMaterials.push(this.getSeperableBlurMaterial(l[u])),this.separableBlurMaterials[u].uniforms.invSize.value=new j(1/r,1/a),r=Math.round(r/2),a=Math.round(a/2);this.compositeMaterial=this.getCompositeMaterial(this.nMips),this.compositeMaterial.uniforms.blurTexture1.value=this.renderTargetsVertical[0].texture,this.compositeMaterial.uniforms.blurTexture2.value=this.renderTargetsVertical[1].texture,this.compositeMaterial.uniforms.blurTexture3.value=this.renderTargetsVertical[2].texture,this.compositeMaterial.uniforms.blurTexture4.value=this.renderTargetsVertical[3].texture,this.compositeMaterial.uniforms.blurTexture5.value=this.renderTargetsVertical[4].texture,this.compositeMaterial.uniforms.bloomStrength.value=e,this.compositeMaterial.uniforms.bloomRadius.value=.1;let c=[1,.8,.6,.4,.2];this.compositeMaterial.uniforms.bloomFactors.value=c,this.bloomTintColors=[new I(1,1,1),new I(1,1,1),new I(1,1,1),new I(1,1,1),new I(1,1,1)],this.compositeMaterial.uniforms.bloomTintColors.value=this.bloomTintColors;let h=Nr;this.copyUniforms=ii.clone(h.uniforms),this.blendMaterial=new Te({uniforms:this.copyUniforms,vertexShader:h.vertexShader,fragmentShader:h.fragmentShader,blending:Qn,depthTest:!1,depthWrite:!1,transparent:!0}),this.enabled=!0,this.needsSwap=!1,this._oldClearColor=new Pt,this.oldClearAlpha=1,this.basic=new Ve,this.fsQuad=new si(null)}dispose(){for(let t=0;t<this.renderTargetsHorizontal.length;t++)this.renderTargetsHorizontal[t].dispose();for(let t=0;t<this.renderTargetsVertical.length;t++)this.renderTargetsVertical[t].dispose();this.renderTargetBright.dispose();for(let t=0;t<this.separableBlurMaterials.length;t++)this.separableBlurMaterials[t].dispose();this.compositeMaterial.dispose(),this.blendMaterial.dispose(),this.basic.dispose(),this.fsQuad.dispose()}setSize(t,e){let i=Math.round(t/2),s=Math.round(e/2);this.renderTargetBright.setSize(i,s);for(let r=0;r<this.nMips;r++)this.renderTargetsHorizontal[r].setSize(i,s),this.renderTargetsVertical[r].setSize(i,s),this.separableBlurMaterials[r].uniforms.invSize.value=new j(1/i,1/s),i=Math.round(i/2),s=Math.round(s/2)}render(t,e,i,s,r){t.getClearColor(this._oldClearColor),this.oldClearAlpha=t.getClearAlpha();let a=t.autoClear;t.autoClear=!1,t.setClearColor(this.clearColor,0),r&&t.state.buffers.stencil.setTest(!1),this.renderToScreen&&(this.fsQuad.material=this.basic,this.basic.map=i.texture,t.setRenderTarget(null),t.clear(),this.fsQuad.render(t)),this.highPassUniforms.tDiffuse.value=i.texture,this.highPassUniforms.luminosityThreshold.value=this.threshold,this.fsQuad.material=this.materialHighPassFilter,t.setRenderTarget(this.renderTargetBright),t.clear(),this.fsQuad.render(t);let o=this.renderTargetBright;for(let l=0;l<this.nMips;l++)this.fsQuad.material=this.separableBlurMaterials[l],this.separableBlurMaterials[l].uniforms.colorTexture.value=o.texture,this.separableBlurMaterials[l].uniforms.direction.value=n.BlurDirectionX,t.setRenderTarget(this.renderTargetsHorizontal[l]),t.clear(),this.fsQuad.render(t),this.separableBlurMaterials[l].uniforms.colorTexture.value=this.renderTargetsHorizontal[l].texture,this.separableBlurMaterials[l].uniforms.direction.value=n.BlurDirectionY,t.setRenderTarget(this.renderTargetsVertical[l]),t.clear(),this.fsQuad.render(t),o=this.renderTargetsVertical[l];this.fsQuad.material=this.compositeMaterial,this.compositeMaterial.uniforms.bloomStrength.value=this.strength,this.compositeMaterial.uniforms.bloomRadius.value=this.radius,this.compositeMaterial.uniforms.bloomTintColors.value=this.bloomTintColors,t.setRenderTarget(this.renderTargetsHorizontal[0]),t.clear(),this.fsQuad.render(t),this.fsQuad.material=this.blendMaterial,this.copyUniforms.tDiffuse.value=this.renderTargetsHorizontal[0].texture,r&&t.state.buffers.stencil.setTest(!0),this.renderToScreen?(t.setRenderTarget(null),this.fsQuad.render(t)):(t.setRenderTarget(i),this.fsQuad.render(t)),t.setClearColor(this._oldClearColor,this.oldClearAlpha),t.autoClear=a}getSeperableBlurMaterial(t){let e=[];for(let i=0;i<t;i++)e.push(.39894*Math.exp(-.5*i*i/(t*t))/t);return new Te({defines:{KERNEL_RADIUS:t},uniforms:{colorTexture:{value:null},invSize:{value:new j(.5,.5)},direction:{value:new j(.5,.5)},gaussianCoefficients:{value:e}},vertexShader:`varying vec2 vUv;
				void main() {
					vUv = uv;
					gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
				}`,fragmentShader:`#include <common>
				varying vec2 vUv;
				uniform sampler2D colorTexture;
				uniform vec2 invSize;
				uniform vec2 direction;
				uniform float gaussianCoefficients[KERNEL_RADIUS];

				void main() {
					float weightSum = gaussianCoefficients[0];
					vec3 diffuseSum = texture2D( colorTexture, vUv ).rgb * weightSum;
					for( int i = 1; i < KERNEL_RADIUS; i ++ ) {
						float x = float(i);
						float w = gaussianCoefficients[i];
						vec2 uvOffset = direction * invSize * x;
						vec3 sample1 = texture2D( colorTexture, vUv + uvOffset ).rgb;
						vec3 sample2 = texture2D( colorTexture, vUv - uvOffset ).rgb;
						diffuseSum += (sample1 + sample2) * w;
						weightSum += 2.0 * w;
					}
					gl_FragColor = vec4(diffuseSum/weightSum, 1.0);
				}`})}getCompositeMaterial(t){return new Te({defines:{NUM_MIPS:t},uniforms:{blurTexture1:{value:null},blurTexture2:{value:null},blurTexture3:{value:null},blurTexture4:{value:null},blurTexture5:{value:null},bloomStrength:{value:1},bloomFactors:{value:null},bloomTintColors:{value:null},bloomRadius:{value:0}},vertexShader:`varying vec2 vUv;
				void main() {
					vUv = uv;
					gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
				}`,fragmentShader:`varying vec2 vUv;
				uniform sampler2D blurTexture1;
				uniform sampler2D blurTexture2;
				uniform sampler2D blurTexture3;
				uniform sampler2D blurTexture4;
				uniform sampler2D blurTexture5;
				uniform float bloomStrength;
				uniform float bloomRadius;
				uniform float bloomFactors[NUM_MIPS];
				uniform vec3 bloomTintColors[NUM_MIPS];

				float lerpBloomFactor(const in float factor) {
					float mirrorFactor = 1.2 - factor;
					return mix(factor, mirrorFactor, bloomRadius);
				}

				void main() {
					gl_FragColor = bloomStrength * ( lerpBloomFactor(bloomFactors[0]) * vec4(bloomTintColors[0], 1.0) * texture2D(blurTexture1, vUv) +
						lerpBloomFactor(bloomFactors[1]) * vec4(bloomTintColors[1], 1.0) * texture2D(blurTexture2, vUv) +
						lerpBloomFactor(bloomFactors[2]) * vec4(bloomTintColors[2], 1.0) * texture2D(blurTexture3, vUv) +
						lerpBloomFactor(bloomFactors[3]) * vec4(bloomTintColors[3], 1.0) * texture2D(blurTexture4, vUv) +
						lerpBloomFactor(bloomFactors[4]) * vec4(bloomTintColors[4], 1.0) * texture2D(blurTexture5, vUv) );
				}`})}};Br.BlurDirectionX=new j(1,0);Br.BlurDirectionY=new j(0,1);var ul=class extends cn{constructor(t){super();let e=Nr;this.textureID="tDiffuse",this.uniforms=ii.clone(e.uniforms),this.material=new Te({uniforms:this.uniforms,vertexShader:e.vertexShader,fragmentShader:e.fragmentShader,blending:Rn}),this.renderTarget=t,this.renderTarget===void 0&&(this.renderTarget=new Ue(1,1,{type:en}),this.renderTarget.texture.name="SavePass.rt"),this.needsSwap=!1,this.fsQuad=new si(this.material)}render(t,e,i){this.uniforms[this.textureID]&&(this.uniforms[this.textureID].value=i.texture),t.setRenderTarget(this.renderTarget),this.clear&&t.clear(),this.fsQuad.render(t)}setSize(t,e){this.renderTarget.setSize(t,e)}dispose(){this.renderTarget.dispose(),this.material.dispose(),this.fsQuad.dispose()}};var Df={name:"OutputShader",uniforms:{tDiffuse:{value:null},toneMappingExposure:{value:1}},vertexShader:`
		precision highp float;

		uniform mat4 modelViewMatrix;
		uniform mat4 projectionMatrix;

		attribute vec3 position;
		attribute vec2 uv;

		varying vec2 vUv;

		void main() {

			vUv = uv;
			gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );

		}`,fragmentShader:`
	
		precision highp float;

		uniform sampler2D tDiffuse;

		#include <tonemapping_pars_fragment>
		#include <colorspace_pars_fragment>

		varying vec2 vUv;

		void main() {

			gl_FragColor = texture2D( tDiffuse, vUv );

			// tone mapping

			#ifdef LINEAR_TONE_MAPPING

				gl_FragColor.rgb = LinearToneMapping( gl_FragColor.rgb );

			#elif defined( REINHARD_TONE_MAPPING )

				gl_FragColor.rgb = ReinhardToneMapping( gl_FragColor.rgb );

			#elif defined( CINEON_TONE_MAPPING )

				gl_FragColor.rgb = CineonToneMapping( gl_FragColor.rgb );

			#elif defined( ACES_FILMIC_TONE_MAPPING )

				gl_FragColor.rgb = ACESFilmicToneMapping( gl_FragColor.rgb );

			#elif defined( AGX_TONE_MAPPING )

				gl_FragColor.rgb = AgXToneMapping( gl_FragColor.rgb );

			#elif defined( NEUTRAL_TONE_MAPPING )

				gl_FragColor.rgb = NeutralToneMapping( gl_FragColor.rgb );

			#endif

			// color space

			#ifdef SRGB_TRANSFER

				gl_FragColor = sRGBTransferOETF( gl_FragColor );

			#endif

		}`};var dl=class extends cn{constructor(){super();let t=Df;this.uniforms=ii.clone(t.uniforms),this.material=new Jo({name:t.name,uniforms:this.uniforms,vertexShader:t.vertexShader,fragmentShader:t.fragmentShader}),this.fsQuad=new si(this.material),this._outputColorSpace=null,this._toneMapping=null}render(t,e,i){this.uniforms.tDiffuse.value=i.texture,this.uniforms.toneMappingExposure.value=t.toneMappingExposure,(this._outputColorSpace!==t.outputColorSpace||this._toneMapping!==t.toneMapping)&&(this._outputColorSpace=t.outputColorSpace,this._toneMapping=t.toneMapping,this.material.defines={},se.getTransfer(this._outputColorSpace)===fe&&(this.material.defines.SRGB_TRANSFER=""),this._toneMapping===Jh?this.material.defines.LINEAR_TONE_MAPPING="":this._toneMapping===Kh?this.material.defines.REINHARD_TONE_MAPPING="":this._toneMapping===Qh?this.material.defines.CINEON_TONE_MAPPING="":this._toneMapping===Ta?this.material.defines.ACES_FILMIC_TONE_MAPPING="":this._toneMapping===jh?this.material.defines.AGX_TONE_MAPPING="":this._toneMapping===tu&&(this.material.defines.NEUTRAL_TONE_MAPPING=""),this.material.needsUpdate=!0),this.renderToScreen===!0?(t.setRenderTarget(null),this.fsQuad.render(t)):(t.setRenderTarget(e),this.clear&&t.clear(t.autoClearColor,t.autoClearDepth,t.autoClearStencil),this.fsQuad.render(t))}dispose(){this.material.dispose(),this.fsQuad.dispose()}};var Di={title:"Saved",kind:"Link",glyph:"link",no:""},Kf="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7",kf={link:"M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71",note:"M16 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8zM15 3v4a2 2 0 0 0 2 2h4M8 13h8M8 17h5",image:"M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM9 11a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM21 15l-3.1-3.1a2 2 0 0 0-2.8 0L6 21",video:"M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM10 8.5v7l6-3.5z",audio:"M9 18V5l12-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM21 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z",pdf:"M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7zM14 2v4a2 2 0 0 0 2 2h4M8 13h2M8 17h8M14 13h2",file:"M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7zM14 2v4a2 2 0 0 0 2 2h4"},Lf={},Qf=n=>Lf[n]||(Lf[n]=new Path2D(kf[n]||kf.file));function Gv(n,t){let e=String(Di.title||"").replace(/\s+/g," ").trim().split(" ").filter(Boolean);if(!e.length)return["Saved"];let i=l=>n.measureText(l).width<=t,s=l=>{if(i(l))return l;for(;l.length>1&&!i(l+"\u2026");)l=l.slice(0,-1).trimEnd();return l+"\u2026"},r=0,a="";for(;r<e.length&&i(a?a+" "+e[r]:e[r]);)a=a?a+" "+e[r]:e[r],r++;let o=e.slice(r).join(" ");if(!a){let l=o.length;for(;l>1&&!i(o.slice(0,l));)l--;a=o.slice(0,l),o=o.slice(l)}return o?[a,s(o)]:[a]}function Wv(n){return(t,e,i)=>{t.save(),t.translate(e-5.5,i-5.5),t.scale(11/24,11/24),t.strokeStyle=n,t.lineWidth=2.2,t.lineCap="round",t.lineJoin="round",t.stroke(Qf(Di.glyph)),t.restore()}}function qv(n,t=186){for(let r=n.length-1;r>=0;r--)n[r][0]||n.splice(r,1);let e=document.createElement("canvas").getContext("2d");e.font=`500 11px ${Re}`;let i=()=>n.reduce((r,[a,o])=>r+e.measureText(a).width+12+(o.icon?14:0)+4,-4);if(!n.length||i()<=t)return;let s=n[0][0];for(;s.length>3&&i()>t;)s=s.slice(0,-1).trimEnd(),n[0][0]=s+"\u2026"}function Xv(){let n=document.createElement("canvas");n.width=600,n.height=800;let t=n.getContext("2d"),e=t.createRadialGradient(300,380,20,300,400,520);e.addColorStop(0,"#1F4378"),e.addColorStop(.6,"#0F2348"),e.addColorStop(1,"#081430"),t.fillStyle=e,t.fillRect(0,0,600,800);let i=7,s=()=>(i=i*16807%2147483647,i/2147483647);for(let r=0;r<90;r++)t.globalAlpha=.15+s()*.5,t.fillStyle="#E8F0FF",t.beginPath(),t.arc(s()*600,s()*800,s()*2+.6,0,7),t.fill();return t.globalAlpha=1,t.save(),t.translate(204,304),t.scale(8,8),t.shadowColor="rgba(243,226,176,.55)",t.shadowBlur=3,t.strokeStyle="#F3E2B0",t.lineWidth=1.5,t.lineCap="round",t.lineJoin="round",t.stroke(Qf(Di.glyph)),t.restore(),n.toDataURL("image/png")}window.addEventListener("error",n=>{try{window.InvApp&&window.InvApp.post("fail",JSON.stringify({m:String(n.message||n)}))}catch{}});var re={1:"#A39E92",3:"#3E8DB5",5:"#E2A23B"},ki={1:"Common",3:"Rare",5:"Legendary"},Se={1:1,3:3,5:5},Ge={1:10,3:40,5:80},$v={1:["#FFFFFF"],3:["#FFFFFF","#3E8DB5"],5:["#FFFFFF","#3E8DB5","#E2A23B"]},Lu="#0B2227";var Yv="#F4F2EF",kn=360,Ln=760,Au=null,Qs=/[?&]film/.test(location.search),jf=!0,we=5,Bi=1,tp=!Qs,ep=0,Tn=0,Uu=1,za=1,Dl=!1,Ks=!1,bl=null,Sn=document.getElementById("scr"),wn=document.getElementById("cam"),Ml=document.getElementById("layer"),kl=document.getElementById("hud"),ka=document.getElementById("shop"),Nu=document.getElementById("over");Sn.addEventListener("scroll",()=>{(Sn.scrollTop||Sn.scrollLeft)&&(Sn.scrollTop=0,Sn.scrollLeft=0)});var Gt=(n,t)=>n+Math.random()*(t-n),Wr=()=>Uu*za,Xs=[];function Cu(){Qs||(Xs=Xs.filter(n=>n.playState!=="finished"||n.effect?.getComputedTiming().iterations===1/0),new Set([...document.getAnimations(),...Xs]).forEach(n=>{n.playbackRate=Wr()}))}function Qt(n,t){let e=parseInt(n.slice(1),16);return`rgba(${e>>16},${e>>8&255},${e&255},${Math.max(0,Math.min(1,t)).toFixed(3)})`}var ie=null,ps=null;function Zv(){if(!ie)try{ie=new(window.AudioContext||window.webkitAudioContext);let n=ie.createDynamicsCompressor();n.threshold.value=-14,n.ratio.value=4,n.connect(ie.destination);let t=ie.createGain();t.gain.value=.6,t.connect(n),ps=ie.createGain(),ps.connect(t);let e=ie.createConvolver(),i=ie.sampleRate*2.4,s=ie.createBuffer(2,i,ie.sampleRate);for(let a=0;a<2;a++){let o=s.getChannelData(a);for(let l=0;l<i;l++)o[l]=(Math.random()*2-1)*Math.pow(1-l/i,3.2)}e.buffer=s;let r=ie.createGain();r.gain.value=.3,ps.connect(e),e.connect(r),r.connect(t)}catch{ie=null}return ie&&ie.state==="suspended"&&ie.resume(),ie}var Or=null;function Uf(){let n=ie.createBuffer(1,ie.sampleRate*2,ie.sampleRate),t=n.getChannelData(0);for(let e=0;e<t.length;e++)t[e]=Math.random()*2-1;return n}var Z={extra:{},tone(n,t,e,i="sine",s=.3,r,a=.008){let o=ie.createOscillator(),l=ie.createGain();o.type=i,o.frequency.setValueAtTime(n,t),r&&o.frequency.exponentialRampToValueAtTime(r,t+e),l.gain.setValueAtTime(0,t),l.gain.linearRampToValueAtTime(s,t+a),l.gain.exponentialRampToValueAtTime(8e-4,t+e),o.connect(l),l.connect(ps),o.start(t),o.stop(t+e+.05)},noise(n,t,e,i,s=.3,r=1.2,a=.3,o="bandpass"){Or=Or||Uf();let l=ie.createBufferSource();l.buffer=Or;let c=ie.createBiquadFilter();c.type=o,c.Q.value=r,c.frequency.setValueAtTime(e,n),c.frequency.exponentialRampToValueAtTime(i,n+t);let h=ie.createGain();h.gain.setValueAtTime(0,n),h.gain.linearRampToValueAtTime(s,n+t*a),h.gain.exponentialRampToValueAtTime(8e-4,n+t),l.connect(c),c.connect(h),h.connect(ps),l.start(n,Math.random()),l.stop(n+t+.05)},play(n,t){if(!tp||Dl||!Zv())return;let e=ie.currentTime+.005,i=Bi,s=(r,a,o=.16,l=1.4)=>{Z.tone(r,a,l*i,"sine",o),Z.tone(r*2.01,a,l*.5*i,"sine",o*.35),Z.tone(r*3.98,a,l*.25*i,"sine",o*.12)};switch(n){case"whoosh":Z.noise(e,.5*i,300,2600,.2,.9);break;case"swoop":Z.noise(e,.38*i,2600,500,.17,1.1);break;case"riser":{let r=(t||1)*i;Z.noise(e,r,250,6500,.2,4,.92),Z.tone(110,e,r,"sawtooth",.03,440,r*.9),Z.tone(111.5,e,r,"sawtooth",.03,446,r*.9);break}case"suck":{Or=Or||Uf();let r=(t||.35)*i,a=ie.createBufferSource();a.buffer=Or;let o=ie.createBiquadFilter();o.type="highpass",o.frequency.setValueAtTime(800,e),o.frequency.exponentialRampToValueAtTime(5e3,e+r);let l=ie.createGain();l.gain.setValueAtTime(8e-4,e),l.gain.exponentialRampToValueAtTime(.32,e+r),l.gain.linearRampToValueAtTime(0,e+r+.02),a.connect(o),o.connect(l),l.connect(ps),a.start(e),a.stop(e+r+.05);break}case"boom":{let r=t??1;Z.tone(72,e,(1+.6*r)*i,"sine",.5+.3*r,26,.004),Z.tone(150,e,.25*i,"triangle",.22*r,55,.002),Z.noise(e,(.7+.5*r)*i,2200,90,.3+.25*r,.5,.01);break}case"crack":Z.noise(e,.1*i,5e3,2500,.5,.6,.02,"highpass"),Z.noise(e,.2*i,1800,900,.25,.8,.02);break;case"step":{let r=t||0,a=[392,523.3,659.3,784][r]||784;s(a,e,.13,1.1),Z.tone(a/4,e,.5*i,"sine",.22,a/4.4,.01),Z.noise(e,.3*i,1200,5e3,.1,1,.05);break}case"thud":Z.tone(110,e,.35*i,"sine",.55,38,.004),Z.noise(e,.14*i,900,250,.35,.7,.02);break;case"clunk":{let r=t??1;Z.tone(95,e,.4*i,"sine",.55*r,40,.003),Z.tone(330,e,.22*i,"triangle",.1*r),Z.tone(487,e,.16*i,"triangle",.07*r),Z.noise(e,.18*i,700,200,.35*r,.6,.02);break}case"stamp":Z.tone(90,e,.35*i,"sine",.65,32,.003),Z.noise(e,.09*i,3200,1200,.45,.8,.02),Z.noise(e+.02,.3*i,600,180,.28,.6,.05);break;case"paper":Z.noise(e,.32*i,1800,5200,.14,.6);break;case"tear":for(let r=0;r<16;r++)Z.noise(e+r*.026*i,.02*i,Gt(2e3,6e3),Gt(2e3,6e3),.22,2,.1);Z.noise(e,.45*i,2500,6e3,.08,.8);break;case"meteor":{let r=(t||1.4)*i;Z.noise(e,r,5e3,500,.22,1.4,.6),Z.tone(1400,e,r,"sine",.05,260,r*.6);break}case"ting":{let r=[1318.5,1760,2349.3][t||0];s(r,e,.14,1.2),Z.noise(e,.25*i,6e3,9e3,.08,3,.05);break}case"zap":{let r=ie.createOscillator(),a=ie.createGain(),o=ie.createBiquadFilter();r.type="square",o.type="bandpass",o.frequency.value=2400,o.Q.value=.7;let l=.26*i;for(let c=0;c<l;c+=.014)r.frequency.setValueAtTime(Gt(60,900),e+c);a.gain.setValueAtTime(.09,e),a.gain.exponentialRampToValueAtTime(.001,e+l),r.connect(o),o.connect(a),a.connect(ps),r.start(e),r.stop(e+l+.02),Z.noise(e,.12*i,7e3,3e3,.14,1,.02,"highpass");break}case"holo":{let r=(t||1)*i,a=ie.createOscillator(),o=ie.createBiquadFilter(),l=ie.createGain(),c=ie.createOscillator(),h=ie.createGain();a.type="sawtooth",a.frequency.value=110,o.type="lowpass",o.frequency.value=900,c.frequency.value=17,h.gain.value=.04,l.gain.setValueAtTime(0,e),l.gain.linearRampToValueAtTime(.06,e+.1),l.gain.linearRampToValueAtTime(0,e+r),c.connect(h),h.connect(l.gain),a.connect(o),o.connect(l),l.connect(ps),a.start(e),c.start(e),a.stop(e+r+.05),c.stop(e+r+.05);break}case"shimmer":for(let r=0;r<14;r++)Z.tone(Gt(2500,6500),e+Gt(0,1.2)*i,.5*i,"sine",.035);break;case"tick":Z.tone(t||1400,e,.07*i,"triangle",.14);break;case"pop":Z.tone(520,e,.14*i,"sine",.3,1200),Z.noise(e,.08*i,2500,4e3,.12,2);break;case"chime":{let r=t||1;(r>=5?[523.3,659.3,784,1046.5,1318.5,1568]:r>=3?[587.3,880,1174.7]:[659.3,987.8]).forEach((o,l)=>s(o,e+l*.07*i,r>=5?.14:.16,r>=5?2.4:1.3)),r>=5&&(Z.tone(130.8,e,2*i,"sine",.2),Z.tone(196,e,2*i,"sine",.08));break}case"hum":Z.tone(110,e,(t||1.2)*i,"sine",.14),Z.tone(165,e,(t||1.2)*i,"sine",.08);break;default:Z.extra[n]?.(e,i,t,s)}}},Jv=document.getElementById("fx"),us=Jv.getContext("2d"),Hr=document.getElementById("fxb").getContext("2d");us.scale(2,2);Hr.scale(2,2);var H=us,Kv={bloom:1,flare:1,rays:1,arc:1},$s=[],Ha=[],Sl=[],wl=[],Va=[],Li=[],fu=performance.now(),$e=0,xs=0,pu={};function ms(n){if(pu[n])return pu[n];let t=document.createElement("canvas");t.width=t.height=64;let e=t.getContext("2d"),i=e.createRadialGradient(32,32,0,32,32,32);return i.addColorStop(0,"rgba(255,255,255,1)"),i.addColorStop(.16,Qt(n,.95)),i.addColorStop(.42,Qt(n,.32)),i.addColorStop(1,Qt(n,0)),e.fillStyle=i,e.fillRect(0,0,64,64),pu[n]=t}var mu=null;function Qv(){if(mu)return mu;let n=document.createElement("canvas");n.width=n.height=64;let t=n.getContext("2d"),e=t.createRadialGradient(32,32,0,32,32,32);return e.addColorStop(0,"rgba(210,214,208,.8)"),e.addColorStop(.5,"rgba(190,196,190,.35)"),e.addColorStop(1,"rgba(180,186,180,0)"),t.fillStyle=e,t.fillRect(0,0,64,64),mu=n}function np(n,t,e){H.beginPath(),H.moveTo(n,t-e),H.quadraticCurveTo(n,t,n+e,t),H.quadraticCurveTo(n,t,n,t+e),H.quadraticCurveTo(n,t,n-e,t),H.quadraticCurveTo(n,t,n,t-e),H.fill()}var ee={out:n=>1-Math.pow(1-n,3),in:n=>n*n*n,inOut:n=>n<.5?4*n*n*n:1-Math.pow(-2*n+2,3)/2,lin:n=>n,in2:n=>n*n};function De(n,t,e,i=ee.out){return new Promise(s=>{let r={};for(let a in t)r[a]=n[a];Li.push({o:n,from:r,to:t,d:e/1e3,t:0,ease:i,res:s})})}function El(n,t,e){let i={fn:n,rate:t,dur:e,t:0,acc:0,dead:!1};return Va.push(i),i}function vs(n){let t=Object.assign({a:1,t:0,dead:!1},n);return Ha.push(t),t}function jv(n,t=300){n&&De(n,{a:0},t,ee.lin).then(()=>{n.dead=!0})}function Ui(n,t,e,i=180,s=6,r=.55,a=0,o=!1){Sl.push({x:n,y:t,color:e,max:i,w:s,dur:r,t:-a,front:o})}function Ga(n=.9,t=.25,e="#FFFFFF"){wl.push({alpha:n,dur:t,color:e,t:0})}function Ys(n){xs=Math.min(1,xs+n)}function Dn(n,t,e,i,s,r={}){for(let a=0;a<i;a++){let o=r.dir!=null?r.dir+Gt(-r.spread,r.spread):Gt(0,Math.PI*2),l=Gt(s*.3,s);$s.push({k:"spark",bk:r.bk,x:n,y:t,vx:Math.cos(o)*l,vy:Math.sin(o)*l,life:Gt(.35,.9)*(r.life||1),t:0,size:Gt(.9,r.size||2.2),color:Math.random()<(r.white??.35)?"#FFFFFF":e,g:r.g??300,drag:r.drag??2.4})}}function ys(n,t,e,i,s,r={}){for(let a=0;a<i;a++)$s.push({k:"glint",bk:r.bk,x:n+Gt(-s,s),y:t+Gt(-s,s)*(r.ry??1),vx:0,vy:r.vy||0,life:Gt(.5,1.1)*(r.life||1),t:-Gt(0,r.delay||0),size:Gt(1,r.size||2.2),color:e,g:0,drag:0,ph:Gt(0,6)})}function ty(n,t,e,i,s,r,a){return El(()=>$s.push({k:"ember",x:Gt(n,t),y:Gt(e,i),vx:Gt(-10,10),vy:Gt(-75,-25),life:Gt(1.4,2.8),t:0,size:Gt(.7,1.7),color:s[Math.random()*s.length|0],g:-4,drag:.3,wob:Gt(1,3),ph:Gt(0,6)}),r,a)}function Nf(n,t,e,i,s){let r=[[n,t],[e,i]],a=Math.hypot(e-n,i-t)*.22;for(let o=0;o<s;o++){let l=[];for(let c=0;c<r.length-1;c++){let[h,u]=r[c],[d,f]=r[c+1];l.push(r[c]);let g=-(f-u),x=d-h,m=Math.hypot(g,x)||1,p=Gt(-a,a);l.push([(h+d)/2+g/m*p,(u+f)/2+x/m*p])}l.push(r[r.length-1]),r=l,a*=.55}return r}function fl(n){H.beginPath(),H.moveTo(n[0][0],n[0][1]);for(let t=1;t<n.length;t++)H.lineTo(n[t][0],n[t][1]);H.stroke()}function ey(n,t){if(n.life&&(n.t+=t,n.t>=n.life))return!1;if("x"in n&&!(Number.isFinite(n.x)&&Number.isFinite(n.y)))return!0;let e=n.life?1-n.t/n.life:1;switch(n.kind){case"glow":{H.globalAlpha=n.a*e;let i=n.r*2;H.drawImage(ms(n.color),n.x-i/2,n.y-i/2*(n.sy||1),i,i*(n.sy||1));break}case"rays":{n.rot=(n.rot||0)+(n.spin||0)*t;let i=n.n||14,s=n.from??-Math.PI,r=n.to??Math.PI;n.j||(n.j=Array.from({length:i},()=>[Gt(.55,1),Gt(-.5,.5),Gt(2,7),Gt(0,6)])),H.globalAlpha=1;for(let a=0;a<i;a++){let[o,l,c,h]=n.j[a],u=s+(r-s)*(a+.5+l*.5)/i+n.rot,d=n.len*o,f=(.6+.4*Math.sin($e*c+h))*n.a*e,g=H.createRadialGradient(n.x,n.y,0,n.x,n.y,d);g.addColorStop(0,Qt(n.color,.6*f)),g.addColorStop(.3,Qt(n.color,.28*f)),g.addColorStop(1,Qt(n.color,0)),H.fillStyle=g;let x=(n.w||.08)*(.6+.4*o);H.beginPath(),H.moveTo(n.x,n.y),H.lineTo(n.x+Math.cos(u-x)*d,n.y+Math.sin(u-x)*d),H.lineTo(n.x+Math.cos(u+x)*d,n.y+Math.sin(u+x)*d),H.closePath(),H.fill()}break}case"pillar":{if(n.w<1||n.h<1)break;let i=n.a*(.86+.14*Math.sin($e*31))*e,s=H.createLinearGradient(n.x-n.w/2,0,n.x+n.w/2,0);s.addColorStop(0,Qt(n.color,0)),s.addColorStop(.28,Qt(n.color,.3*i)),s.addColorStop(.45,Qt("#FFFFFF",.85*i)),s.addColorStop(.55,Qt("#FFFFFF",.85*i)),s.addColorStop(.72,Qt(n.color,.3*i)),s.addColorStop(1,Qt(n.color,0)),H.fillStyle=s;let r=16;for(let o=0;o<r;o++)H.globalAlpha=Math.pow(1-o/r,1.2),H.fillRect(n.x-n.w/2,n.y-n.h*(o+1)/r,n.w,n.h/r+.6);H.globalAlpha=i;let a=n.w*2.8;H.drawImage(ms(n.color),n.x-a/2,n.y-a*.18,a,a*.36);break}case"flare":{H.globalAlpha=n.a*e;let i=ms(n.color);H.drawImage(i,n.x-n.len,n.y-n.th,n.len*2,n.th*2),H.drawImage(ms("#FFFFFF"),n.x-n.len*.55,n.y-n.th*.3,n.len*1.1,n.th*.6);let s=n.core||70;H.drawImage(i,n.x-s/2,n.y-s/2,s,s);break}case"bolt":{(!n.pts||Math.random()<.45)&&(n.pts=Nf(n.x1,n.y1,n.x2,n.y2,5));let i=n.a*e*(Math.random()<.2?.3:1);H.lineJoin="round",H.globalAlpha=i*.45,H.strokeStyle=n.color,H.lineWidth=4.5,fl(n.pts),H.globalAlpha=i,H.strokeStyle="#FFFFFF",H.lineWidth=1.2,fl(n.pts);break}case"comet":{if(n.follow){if(!n.follow.isConnected)return!1;let[a,o]=Ba(n.follow);n.x=a,n.y=o}for(n.trail.push([n.x,n.y]);n.trail.length>(n.n||24);)n.trail.shift();let i=n.trail,s=i.length;H.lineCap="round";for(let a=1;a<s;a++){let o=a/s;H.globalAlpha=Math.pow(o,1.6)*n.a,H.strokeStyle=n.color,H.lineWidth=n.w*o*2.4,H.beginPath(),H.moveTo(i[a-1][0],i[a-1][1]),H.lineTo(i[a][0],i[a][1]),H.stroke()}for(let a=1;a<s;a++){let o=a/s;H.globalAlpha=o*o*n.a,H.strokeStyle="#FFFFFF",H.lineWidth=n.w*o*.7,H.beginPath(),H.moveTo(i[a-1][0],i[a-1][1]),H.lineTo(i[a][0],i[a][1]),H.stroke()}H.globalAlpha=n.a;let r=n.w*14;H.drawImage(ms(n.color),n.x-r/2,n.y-r/2,r,r),H.fillStyle="#FFFFFF",np(n.x,n.y,n.w*3.2*(.85+.15*Math.sin($e*40)));break}case"crack":{if(!n.br){n.br=[];for(let s=0;s<n.n;s++){let r=-Math.PI/2+s/n.n*Math.PI*2+Gt(-.3,.3),a=Gt(.55,1)*n.R;n.br.push(Nf(n.x+Gt(-5,5),n.y+Gt(-5,5),n.x+Math.cos(r)*a,n.y+Math.sin(r)*a*1.1,4))}}let i=Math.max(0,Math.min(1,n.p??1));H.lineJoin="round",H.lineCap="round";for(let s of n.br){let r=Math.max(2,Math.ceil(s.length*i)),a=s.slice(0,r);H.globalAlpha=n.a*.45,H.strokeStyle=n.color,H.lineWidth=5,fl(a),H.globalAlpha=n.a,H.strokeStyle="#FFFFFF",H.lineWidth=1.2,fl(a)}break}case"arc":{H.globalAlpha=n.a*e,H.strokeStyle=n.color,H.lineCap="round",H.lineWidth=n.w*(.4+.6*e),H.beginPath(),H.ellipse(n.x,n.y,n.rx,n.ry,0,n.a0,n.a1),H.stroke();break}case"bloom":{let i=1-e,s=n.a*e*e,r=n.R*(.6+.4*i),a=H.createRadialGradient(n.x,n.y,0,n.x,n.y,r);a.addColorStop(0,Qt("#FFFFFF",s)),a.addColorStop(.25,Qt(n.color,s*.75)),a.addColorStop(1,Qt(n.color,0)),H.globalAlpha=1,H.fillStyle=a,H.fillRect(0,0,kn,Ln);break}}return!n.dead}function ny(n,t){if(n.t+=t,n.t<0)return!0;if(n.t>=n.life)return!1;let e=Math.exp(-n.drag*t);return n.vx*=e,n.vy*=e,n.vy+=n.g*t,n.wob&&(n.vx+=Math.sin($e*n.wob+n.ph)*24*t),n.x+=n.vx*t,n.y+=n.vy*t,n.vr&&(n.rot+=n.vr*t),!0}var Bf={smoke:1,shard:1,ink:1};function Of(n){if(n.t<0)return;let t=1-n.t/n.life;switch(n.k){case"spark":{let e=Math.hypot(n.vx,n.vy)||1,i=Math.min(36,e*.045+1.5);H.globalAlpha=Math.min(1,t*1.6),H.strokeStyle=n.color,H.lineCap="round",H.lineWidth=n.size*(.4+.6*t),H.beginPath(),H.moveTo(n.x-n.vx/e*i,n.y-n.vy/e*i),H.lineTo(n.x,n.y),H.stroke(),H.globalAlpha*=.55;let s=n.size*5;H.drawImage(ms(n.color),n.x-s/2,n.y-s/2,s,s);break}case"mote":case"ember":{let e=n.inward?Math.min(1,(1-t)*1.6):Math.min(1,t*1.5),i=n.k==="ember"?.55+.45*Math.sin($e*11+n.ph):1;H.globalAlpha=e*i;let s=n.size*6;H.drawImage(ms(n.color),n.x-s/2,n.y-s/2,s,s);break}case"glint":{let e=Math.max(0,Math.sin(Math.PI*n.t/n.life)),i=e*(.75+.25*Math.sin($e*22+n.ph));H.globalAlpha=i,H.fillStyle="#FFFFFF",np(n.x,n.y,n.size*3.4*i+.4);let s=n.size*9*i;H.drawImage(ms(n.color),n.x-s/2,n.y-s/2,s,s);break}case"smoke":{let e=n.size*(1+(1-t)*2);H.globalAlpha=n.a*t,H.drawImage(Qv(),n.x-e/2,n.y-e/2,e,e);break}case"shard":{H.globalAlpha=Math.min(1,t*2),H.fillStyle=n.color,H.save(),H.translate(n.x,n.y),H.rotate(n.rot),H.scale(Math.cos(n.rot*1.7),1),H.fillRect(-n.size,-n.size*.6,n.size*2,n.size*1.2),H.restore();break}case"ink":{H.globalAlpha=Math.min(1,t*2.5),H.fillStyle=n.color,H.beginPath(),H.arc(n.x,n.y,n.size*(.6+.4*t),0,7),H.fill();break}}}var gu=n=>Math.sin(n)*.5+Math.sin(n*2.13+1.3)*.3+Math.sin(n*4.7+.4)*.2,Ru=new Set;function Ll(n){Ru.add(n)}function Zs(n,t){Ll(e=>{if(t-=e,t<=0)return n(),!1})}function ip(n){let t=Math.min(.05,(n-fu)/1e3);fy(n-fu),fu=n,sp(t),requestAnimationFrame(ip)}function sp(n){let t=n/Bi*Wr();$e+=t;for(let s of[...Ru])s(n*1e3)===!1&&Ru.delete(s);Qs&&(Xs=Xs.filter(s=>{if(s.playState==="idle"||s.playState==="finished")return!1;s.currentTime=(s.currentTime||0)+n*1e3*Wr();let r=s.effect.getComputedTiming().endTime;return s.currentTime>=r?(s.finish(),!1):!0}));for(let s=Li.length-1;s>=0;s--){let r=Li[s];r.t+=t;let a=Math.min(1,r.t/r.d),o=r.ease(a);for(let l in r.to)r.o[l]=r.from[l]+(r.to[l]-r.from[l])*o;a>=1&&(Li.splice(s,1),r.res())}Va=Va.filter(s=>{if(s.dead)return!1;for(s.t+=t,s.acc+=t*s.rate;s.acc>1;)s.fn(),s.acc--;return s.t<s.dur}),xs=Math.max(0,xs-t*1.5);let e=xs*xs;wn.style.translate=e>.001?`${(16*e*gu($e*34)).toFixed(2)}px ${(16*e*gu($e*31+7)).toFixed(2)}px`:"",wn.style.rotate=e>.001?`${(2.4*e*gu($e*23+3)).toFixed(3)}deg`:"";let i=s=>{s(Hr),s(us)};i(s=>{s.clearRect(0,0,kn,Ln),s.globalCompositeOperation="source-over"}),$s=$s.filter(s=>{let r=ny(s,t);return r&&Bf[s.k]&&(H=s.bk?Hr:us,Of(s)),r}),i(s=>{s.globalCompositeOperation="lighter"}),Ha=Ha.filter(s=>(H=s.bk??Kv[s.kind]?Hr:us,ey(s,t))),Sl=Sl.filter(s=>{if(s.t+=t,s.t<0)return!0;let r=s.t/s.dur;if(r>=1)return!1;let a=1-Math.pow(1-r,3),o=6+a*s.max;return H=s.front?us:Hr,H.strokeStyle=s.color,H.globalAlpha=(1-r)*.35,H.lineWidth=s.w*3*(1-r)+1,H.beginPath(),H.arc(s.x,s.y,o,0,7),H.stroke(),H.globalAlpha=(1-r)*.95,H.lineWidth=s.w*(1-r)+.6,H.beginPath(),H.arc(s.x,s.y,o,0,7),H.stroke(),H.strokeStyle="#FFFFFF",H.globalAlpha=(1-r)*.7,H.lineWidth=Math.max(.6,s.w*.3*(1-r)),H.beginPath(),H.arc(s.x,s.y,o,0,7),H.stroke(),!0});for(let s of $s)Bf[s.k]||(H=s.bk?Hr:us,Of(s));i(s=>{s.globalCompositeOperation="source-over",s.globalAlpha=1}),H=us,wl=wl.filter(s=>{s.t+=t;let r=s.t/s.dur;return r>=1?!1:(H.globalAlpha=s.alpha*(1-r)*(1-r),H.fillStyle=s.color,H.fillRect(0,0,kn,Ln),!0)}),H.globalAlpha=1,Au&&Au(t)}Qs?window.__film=async(n,t=1,e=!0)=>{for(let i=0;i<t;i++)jf=e&&i===t-1,sp(n/1e3),await new Promise(s=>setTimeout(s,0))}:requestAnimationFrame(ip);var j_=matchMedia("(prefers-reduced-motion: reduce)").matches,Ms=class extends Error{};function Ni(n){if(n!==Tn)throw new Ms}var ke=(n,t)=>new Promise((e,i)=>{let s=n*Bi;Ll(r=>{if(t!==Tn)return i(new Ms),!1;if(s-=r*Wr(),s<=0)return e(),!1})});function _l(n){if(Dl)return;za=0,Cu();let t=Tn;Zs(()=>{za=1,t===Tn&&Cu()},n*Bi)}function It(n,t,e,i={}){let s=n.animate(t,{duration:e*Bi,fill:"forwards",easing:i.ease||"cubic-bezier(.2,.8,.2,1)",delay:(i.delay||0)*Bi,iterations:i.iter||1,direction:i.dir||"normal"});return Qs?s.pause():s.playbackRate=Wr(),Xs.push(s),s}function We(n){let t=document.createElement("template");return t.innerHTML=n.trim(),t.content.firstElementChild}function Vr(n,t){let e=We(n);return t?Ml.insertBefore(e,t):Ml.appendChild(e),e}function iy(n){let t=We(n);return Nu.appendChild(t),t}function di(n){let t=We(n);return kl.appendChild(t),t}function Ba(n){let t=n.getBoundingClientRect(),e=Sn.getBoundingClientRect(),i=e.width/360,s=(t.left+t.width/2-e.left)/i,r=(t.top+t.height/2-e.top)/i;if(wn.contains(n)){let a=getComputedStyle(wn).transform;if(a&&a!=="none"){let o=new DOMMatrix(a).inverse().transformPoint(new DOMPoint(s-180,r-342));s=o.x+180,r=o.y+342}}return[s,r]}function Pu(n,t,e=0,i="cubic-bezier(.3,.7,.3,1)"){let s=getComputedStyle(wn).transform;return wn.getAnimations().forEach(r=>{r.commitStyles?.(),r.cancel()}),It(wn,[{transform:s==="none"?"none":s},{transform:`translateY(${e}px) scale(${n})`}],t,{ease:i})}function rp(n){let t=Vr(`<div style="position:absolute;inset:0;background:${n};opacity:0"></div>`);return It(t,[{opacity:0},{opacity:1}],420),t}function Bu(n,t=300){n&&It(n,[{opacity:getComputedStyle(n).opacity},{opacity:0}],t)}function sy(n,t,e,i=640,s){let r=Vr(`<div style="position:absolute;left:${n}px;top:${t}px;width:${i}px;height:${i}px;margin:${-i/2}px 0 0 ${-i/2}px;opacity:0;mix-blend-mode:screen">
    <div class="ra" style="position:absolute;inset:0;border-radius:50%;background:repeating-conic-gradient(from 0deg, transparent 0deg 8deg, ${Qt(e,.3)} 10deg 13deg, transparent 15deg 24deg);-webkit-mask:radial-gradient(closest-side, transparent 14%, #000 26%, transparent 100%);mask:radial-gradient(closest-side, transparent 14%, #000 26%, transparent 100%)"></div>
    <div class="rb" style="position:absolute;inset:0;border-radius:50%;background:repeating-conic-gradient(from 6deg, transparent 0deg 13deg, rgba(255,255,255,.14) 15deg 17deg, transparent 19deg 31deg);-webkit-mask:radial-gradient(closest-side, transparent 12%, #000 22%, transparent 90%);mask:radial-gradient(closest-side, transparent 12%, #000 22%, transparent 90%)"></div>
    <div style="position:absolute;inset:28%;border-radius:50%;background:radial-gradient(${Qt(e,.6)}, transparent 70%)"></div></div>`,s);return It(r.querySelector(".ra"),[{transform:"rotate(0)"},{transform:"rotate(360deg)"}],26e3,{iter:1/0,ease:"linear"}),It(r.querySelector(".rb"),[{transform:"rotate(0)"},{transform:"rotate(-360deg)"}],34e3,{iter:1/0,ease:"linear"}),r}var qr=document.getElementById("gl"),Ne=new Ho({canvas:qr,antialias:!1,alpha:!0,powerPreference:"high-performance"}),ry=matchMedia("(pointer: coarse)").matches,Xn=Math.min((window.devicePixelRatio||1)*Math.min(innerWidth/360,innerHeight/760),2.25);Ne.setPixelRatio(Xn);Ne.setSize(kn,Ln,!1);Ne.setClearColor(0,0);Ne.toneMapping=Ta;Ne.toneMappingExposure=1.05;var fi=new ma,qa=new Qe(30,kn/Ln,.1,80);qa.position.set(0,0,14.182);var Ss=new He;fi.add(Ss);var ap=new tl("#FFF4E6",2.2);ap.position.set(-3,4,9);fi.add(ap);fi.add(new el("#CFE8E3",.5));var ay=new Fr(Ne),oy=Array.from({length:4},()=>{let n=new jo("#FFFFFF",0,6,1.6);return fi.add(n),n}),Oi={on:!1,tick:new Set},ly=n=>(n-180)/100,cy=n=>(380-n)/100;function hy(n){let t=new ma;t.add(new Ot(new is(20,24,12),new Ve({color:"#06171A",side:je})));let e=(s,r,a,o,l,c,h)=>{let u=new Ot(new ei(s,r),new Ve({color:new Pt(c).multiplyScalar(h),side:vn}));u.position.set(a,o,l),u.lookAt(0,0,0),t.add(u)};e(7,1.1,0,9,4,"#FFFFFF",7),e(1.1,11,-9,1,2,"#A7D3CC",3.2),e(1.1,11,9,-1,2,n,4),e(7,5,0,-1,10,"#FFFFFF",.14),e(3,1.2,5,6,-6,"#FFFFFF",2.2),e(1.2,6,-4,-7,6,n,1.6);let i=ay.fromScene(t,.02).texture;return t.traverse(s=>{s.geometry?.dispose(),s.material?.dispose()}),i}var zf={};function op(n){fi.environment=zf[n]||(zf[n]=hy(re[n]))}var En=new cl(Ne,new Ue(kn*Xn,Ln*Xn,{type:en,samples:ry?0:4}));En.setPixelRatio(Xn);En.setSize(kn,Ln);En.addPass(new hl(fi,qa));var uy=new Hs({uniforms:{tDiffuse:{value:null},uProbe:{value:typeof opts<"u"&&opts.probe?1:0}},vertexShader:"varying vec2 vUv; void main(){ vUv = uv; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.); }",fragmentShader:"uniform sampler2D tDiffuse; uniform float uProbe; varying vec2 vUv; bool bad(float x){ return !(x >= -1e4 && x <= 1e4); } void main(){ vec4 c = texture2D(tDiffuse, vUv); if (bad(c.r) || bad(c.g) || bad(c.b) || bad(c.a)) c = uProbe > .5 ? vec4(1., 0., 1., 1.) : vec4(0.); gl_FragColor = max(c, vec4(0.)); }"});En.addPass(uy);var lp=new ul(new Ue(kn*Xn,Ln*Xn,{type:en}));En.addPass(lp);var dy=new Br(new j(kn,Ln),.6,.5,.82);En.addPass(dy);var cp=new Hs({uniforms:{tDiffuse:{value:null},tScene:{value:null}},vertexShader:"varying vec2 vUv; void main(){ vUv = uv; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.); }",fragmentShader:"uniform sampler2D tDiffuse, tScene; varying vec2 vUv; void main(){ vec4 a = texture2D(tScene, vUv); vec3 c = texture2D(tDiffuse, vUv).rgb; vec3 g = max(c - a.rgb, 0.); gl_FragColor = vec4(c, clamp(a.a + max(g.r, max(g.g, g.b)), 0., 1.)); }"});cp.uniforms.tScene.value=lp.renderTarget.texture;En.addPass(cp);En.addPass(new dl);var zr={n:0,slow:0};function fy(n){if(!Oi.on||Qs||document.hidden||(zr.n++,n>21&&zr.slow++,zr.n<40))return;let t=zr.slow>16;zr.n=zr.slow=0,t&&Xn>1.26&&(Xn=Math.max(1.25,Math.round((Xn-.25)*100)/100),Ne.setPixelRatio(Xn),Ne.setSize(kn,Ln,!1),En.setPixelRatio(Xn))}function py(){Oi.on||(Oi.on=!0,qr.style.opacity="1")}var Hf=new Map;function my(n){let t=Ne.properties.get(n).currentProgram?.cacheKey;t&&!Hf.has(t)?Hf.set(t,n):n.dispose()}function hp(){oy.forEach(n=>{fi.add(n),n.intensity=0,n.used=!1}),Ss.children.slice().forEach(n=>{Ss.remove(n),n.traverse(t=>{t.geometry?.dispose(),[].concat(t.material||[]).forEach(e=>{for(let i in e){let s=e[i];s&&s.isTexture&&!s.keep&&s.dispose()}my(e)})})}),Oi.tick.clear(),ws.clear(),Hy.b=null,Oi.on=!1,qr.getAnimations().forEach(n=>n.cancel()),qr.style.opacity="0",fi.background=null,Ne.setRenderTarget(null),Ne.clear()}function gy(n){if(Oi.on){if(ws.forEach(t=>Sy(t,n)),Oi.tick.forEach(t=>t(n)),Qs){jf&&En.render();return}if(!xu){if(xy()){Ne.setRenderTarget(En.readBuffer);let t;try{t=Ne.compileAsync(Ss,qa,fi)}catch{}if(Ne.setRenderTarget(null),t){xu=Promise.race([t,new Promise(e=>setTimeout(e,3e3))]).finally(()=>{xu=null});return}}En.render()}}}var xu=null;function xy(){let n=!1;return Ss.traverse(t=>{if(!(n||!(t.isMesh||t.isPoints||t.isLine||t.isSprite))){for(let e of[].concat(t.material||[]))if(!Ne.properties.get(e).currentProgram){n=!0;return}}}),n}function Xr(n,t,e,i,s=-t/2,r=-e/2){return n.moveTo(s+i,r),n.lineTo(s+t-i,r),n.quadraticCurveTo(s+t,r,s+t,r+i),n.lineTo(s+t,r+e-i),n.quadraticCurveTo(s+t,r+e,s+t-i,r+e),n.lineTo(s+i,r+e),n.quadraticCurveTo(s,r+e,s,r+e-i),n.lineTo(s,r+i),n.quadraticCurveTo(s,r,s+i,r),n}function La(n,t,e,i=!1){let s=new Dr(Xr(new Xe,n,t,e),12),r=s.attributes.position,a=s.attributes.uv;for(let o=0;o<r.count;o++){let l=r.getX(o)/n+.5,c=r.getY(o)/t+.5;a.setXY(o,i?1-l:l,c)}return s}function Tt(n,t=!0){let e=new qo(n);return t&&(e.colorSpace=Ke),e.anisotropy=Ne.capabilities.getMaxAnisotropy(),e}function Pe(n,t){let e=document.createElement("canvas");return e.width=n,e.height=t,e.getContext("2d",{willReadFrequently:!0}),e}function Nt(n,t,e,i,s,r){n.beginPath(),n.moveTo(t+r,e),n.arcTo(t+i,e,t+i,e+s,r),n.arcTo(t+i,e+s,t,e+s,r),n.arcTo(t,e+s,t,e,r),n.arcTo(t,e,t+i,e,r),n.closePath()}var Re='"Google Sans Flex", system-ui, sans-serif',un=new Image;un.src=Kf;var _s=new wa().load(Kf);_s.colorSpace=Ke;_s.keep=!0;function ci(n,t,e,i,s){n.save(),n.translate(t-i/2,e-i/2),n.scale(i/48,i/48);let r=(a,o)=>{n.fillStyle=o,n.fill(new Path2D(a))};r("M24 3 L42 18 L24 45 L6 18 Z",s[3]),r("M6 18 L15 9 L24 3 L33 9 L42 18 Z",s[1]),r("M15 9 L24 18 L33 9 L24 3Z",s[0]),r("M6 18 L24 18 L24 45 Z",s[2]),r("M24 18 L42 18 L24 45 Z",s[4]),r("M15 9 L24 18 L6 18Z",s[2]),r("M33 9 L42 18 L24 18Z",s[3]),n.strokeStyle="rgba(255,255,255,.5)",n.lineWidth=.8,n.stroke(new Path2D("M24 3 L42 18 L24 45 L6 18 Z")),n.restore()}function Js(n,t,e,i,s){n.beginPath();for(let r=0;r<10;r++){let a=-Math.PI/2+r*Math.PI/5,o=r%2?i*.45:i;n.lineTo(t+Math.cos(a)*o,e+Math.sin(a)*o)}n.closePath(),n.fillStyle=s,n.fill()}function gs(n,t,e,i,s={}){n.font=`${s.weight||500} ${s.size||10}px ${Re}`;let r=n.measureText(i).width,a=s.pad??7,o=s.icon?(s.size||10)+3:0,l=r+a*2+o,c=s.h||18;return Nt(n,t,e,l,c,c/2),n.fillStyle=s.bg||"rgba(244,242,239,.09)",n.fill(),s.stroke&&(n.strokeStyle=s.stroke,n.lineWidth=1,n.stroke()),s.icon&&s.icon(n,t+a+(s.size||10)/2,e+c/2),n.fillStyle=s.fg||"#CFE0DE",n.textBaseline="middle",n.fillText(i,t+a+o,e+c/2+.5),l}var Ii=2,ds=2.84,Ca=.16,Vs=.035,bs={5:["#FFF3D1","#E2A23B","#8A5A14","#F6CF7A","#6E4510"],3:["#E6F3FA","#3E8DB5","#1D4A63","#86C4E0","#173A4E"],1:["#F1EDE4","#A39E92","#5E5A51","#D4CEC1","#4A4740"]},Fi={teal:["#F4F2EF","#CFE7E2","#A7D3CC","#6FA7A0","#3F7A82"],gold:["#FFF3D1","#F2C46A","#E2A23B","#B07A22","#8A5A14"],blue:["#EAF4FA","#9CCBE3","#3E8DB5","#2A6A8C","#1D4A63"]},Tl={1:"teal",3:"blue",5:"gold"};function vy(n,t=[],e=0){let s=Pe(800,1136),r=s.getContext("2d");r.scale(4,4);let a=re[n],o=bs[n];Nt(r,0,0,200,284,16);let l=r.createLinearGradient(0,0,200,284);[o[0],o[1],o[2],o[3],o[1],o[4],o[0]].forEach((f,g)=>l.addColorStop(g/6,f)),r.fillStyle=l,r.fill(),r.save(),Nt(r,7,7,186,270,11),r.clip(),r.fillStyle=Lu,r.fillRect(0,0,200,284);let c=r.createRadialGradient(100,-10,0,100,-10,210);c.addColorStop(0,Qt(a,.34)),c.addColorStop(1,Qt(a,0)),r.fillStyle=c,r.fillRect(0,0,200,284),r.strokeStyle=Qt("#A7D3CC",.07),r.lineWidth=.5;for(let f=0;f<26;f++){r.beginPath();for(let g=0;g<=200;g+=2){let x=150+f*6+Math.sin(g/13+f*.7)*5+Math.sin(g/31-f)*3;g?r.lineTo(g,x):r.moveTo(g,x)}r.stroke()}r.restore(),r.save(),Nt(r,7,7,186,270,11),r.lineWidth=1,r.strokeStyle="rgba(255,255,255,.18)",r.stroke(),r.restore(),r.font=`700 12px ${Re}`,r.textBaseline="middle",r.fillStyle=a,r.fillText(ki[n],16,21);for(let f=0;f<Se[n];f++)Js(r,184-(Se[n]-1-f)*13,21,5.6,f<e?a:"rgba(244,242,239,.14)");Nt(r,11,32,178,146,11),r.fillStyle="#030D10",r.fill();let h=r.createLinearGradient(0,32,0,60);h.addColorStop(0,"rgba(0,0,0,.7)"),h.addColorStop(1,"rgba(0,0,0,0)"),r.fillStyle=h,r.fill(),r.fillStyle="#F4F2EF",r.font=`500 16px ${Re}`,r.textBaseline="alphabetic",r.fillText("Rangefinder camera,",13,199),r.fillText("1970s",13,218);let u=12;u+=gs(r,u,228,"Camera",{icon:(f,g,x)=>{f.strokeStyle="#CFE0DE",f.lineWidth=1.2,Nt(f,g-5,x-3.5,10,7.5,1.6),f.stroke(),f.beginPath(),f.arc(g,x,2,0,7),f.stroke()}})+4,u+=gs(r,u,228,"No. 024")+4,gs(r,u,228,"+"+Ge[n],{fg:a,icon:(f,g,x)=>ci(f,g,x,10,Fi[Tl[n]])}),u=12;for(let f=0;f<t.length;f++){let g=t[f],x=g.k==="space",m=(x?"":"#")+g.label;r.font=`${x?600:500} 10px ${Re}`;let p=r.measureText(m).width+14+(x?13:0);if(u+p>188){gs(r,u,252,"+"+(t.length-f),{bg:"rgba(244,242,239,.14)",fg:"#E8ECEB"});break}u+=gs(r,u,252,m,x?{bg:"#F4F2EF",fg:"#0E333A",weight:600,icon:(T,M,y)=>{T.fillStyle=g.col,T.beginPath(),T.arc(M,y,3,0,7),T.fill()}}:{bg:"rgba(244,242,239,.14)",fg:"#E8ECEB"})+4}return s}function yy(n){let e=Pe(400,568),i=Pe(400,568),s=e.getContext("2d"),r=i.getContext("2d");if(s.scale(2,2),r.scale(2,2),s.fillStyle="rgb(0,60,255)",s.fillRect(0,0,200,284),Nt(s,7,7,186,270,11),s.fillStyle=n>=5?"rgb(0,95,90)":"rgb(0,165,0)",s.fill(),s.fillStyle="rgb(0,70,230)",s.font=`700 12px ${Re}`,s.textBaseline="middle",s.fillText(ki[n],16,21),r.fillStyle="#000",r.fillRect(0,0,200,284),n>=3&&(r.fillStyle="#FFF",r.fillRect(0,0,200,284),Nt(r,7,7,186,270,11),r.fillStyle="#000",r.fill()),n>=5){r.save(),Nt(r,7,7,186,270,11),r.clip(),r.strokeStyle="rgba(255,255,255,.55)",r.lineWidth=1.1;for(let o=-30;o<60;o++)r.beginPath(),r.moveTo(o*7,0),r.lineTo(o*7-160,284),r.stroke();let a=r.createRadialGradient(100,120,0,100,120,170);a.addColorStop(0,"rgba(255,255,255,.5)"),a.addColorStop(1,"rgba(255,255,255,.12)"),r.fillStyle=a,r.fillRect(0,0,200,284),r.restore(),Nt(r,11,32,178,146,11),r.fillStyle="#000",r.fill()}return{orm:Tt(e,!1),foil:Tt(i,!1)}}function Vf(n){let e=Pe(800,1136),i=e.getContext("2d");i.scale(4,4);let s=bs[n];Nt(i,0,0,200,284,16);let r=i.createLinearGradient(200,0,0,284);[s[0],s[1],s[2],s[3],s[1],s[4],s[0]].forEach((l,c)=>r.addColorStop(c/6,l)),i.fillStyle=r,i.fill(),i.save(),Nt(i,7,7,186,270,11),i.clip();let a=i.createRadialGradient(100,130,0,100,130,190);a.addColorStop(0,"#1E555D"),a.addColorStop(.7,Lu),i.fillStyle=a,i.fillRect(0,0,200,284),i.translate(100,130),i.strokeStyle="rgba(167,211,204,.22)",i.lineWidth=.5;for(let l=0;l<48;l++){i.beginPath();for(let c=0;c<=361;c+=3){let h=c*Math.PI/180,u=52+22*Math.sin(h*6+l*.13)+l*.9;c?i.lineTo(Math.cos(h)*u,Math.sin(h)*u):i.moveTo(Math.cos(h)*u,Math.sin(h)*u)}i.stroke()}for(let l=0;l<3;l++)i.beginPath(),i.arc(0,0,44-l*4,0,7),i.strokeStyle=l?"rgba(167,211,204,.35)":"rgba(244,242,239,.6)",i.lineWidth=l?.6:1.2,i.stroke();i.font=`500 7px ${Re}`,i.fillStyle="rgba(207,231,226,.85)";let o="INVENTORY \xB7 KEPT FOR YOU \xB7 ".toLowerCase().repeat(2);for(let l=0;l<o.length;l++){let c=-Math.PI/2+l/o.length*Math.PI*2;i.save(),i.rotate(c+Math.PI/2),i.fillText(o[l],-2,-33),i.restore()}return ci(i,0,-1,40,Fi.teal),i.restore(),i.fillStyle="#CFE7E2",i.font=`500 14px ${Re}`,i.textAlign="center",i.fillText("Inventory",100,250),i.textAlign="left",e}function _y(){let n=Pe(400,568),t=n.getContext("2d");t.scale(2,2),t.fillStyle="rgb(0,60,255)",t.fillRect(0,0,200,284),Nt(t,7,7,186,270,11),t.fillStyle="rgb(0,170,0)",t.fill(),t.save(),t.translate(100,130),t.strokeStyle="rgb(0,70,200)",t.lineWidth=.7;for(let e=0;e<48;e+=2){t.beginPath();for(let i=0;i<=361;i+=3){let s=i*Math.PI/180,r=52+22*Math.sin(s*6+e*.13)+e*.9;i?t.lineTo(Math.cos(s)*r,Math.sin(s)*r):t.moveTo(Math.cos(s)*r,Math.sin(s)*r)}t.stroke()}return t.beginPath(),t.arc(0,0,44,0,7),t.lineWidth=2,t.stroke(),t.restore(),Tt(n,!1)}var by="varying vec2 vUv; varying vec3 vN; varying vec3 vP; void main(){ vUv = uv; vec4 wp = modelMatrix * vec4(position, 1.); vP = wp.xyz; vN = normalize(mat3(modelMatrix) * normal); gl_Position = projectionMatrix * viewMatrix * wp; }",My=`uniform sampler2D uMask; uniform float uAmt, uSheen, uSheenA, uDens; varying vec2 vUv; varying vec3 vN; varying vec3 vP;
float h21(vec2 p){ p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }
vec3 hue(float h){ return clamp(abs(mod(h * 6. + vec3(0., 4., 2.), 6.) - 3.) - 1., 0., 1.); }
void main(){
  float m = texture2D(uMask, vUv).r;
  vec3 V = normalize(cameraPosition - vP), N = normalize(vN);
  vec2 gp = vUv * vec2(64., 91.) * uDens; vec2 id = floor(gp); vec2 f = fract(gp) - .5;
  float r1 = h21(id), r2 = h21(id + 7.13), r3 = h21(id + 3.71);
  float d = length(f - (vec2(r1, r2) - .5) * .6);
  vec3 n = normalize(N + vec3((r1 - .5) * 1.3, (r2 - .5) * 1.3, 0.));
  vec3 L1 = normalize(vec3(-.5, .8, .6)), L2 = normalize(vec3(.7, -.3, .7));
  float s = pow(max(dot(n, normalize(L1 + V)), 0.), 220.) + pow(max(dot(n, normalize(L2 + V)), 0.), 220.);
  float spark = s * smoothstep(.2, 0., d) * step(.5, r3);
  vec3 col = mix(vec3(1.), hue(r3 * 3. + dot(V, vec3(.8, .4, 0.))), .55);
  float sx = vUv.x * .8 + (1. - vUv.y) * .6 - uSheen; float sw = (exp(-pow(sx * 16., 2.)) * .7 + exp(-pow(sx * 5., 2.)) * .1) * uSheenA;
  gl_FragColor = vec4(col * spark * 3.2 * m * uAmt + vec3(sw), 1.);
}`,up=(()=>{let n=Pe(256,340),t=n.getContext("2d");t.filter="blur(34px)",Nt(t,60,64,136,212,30),t.fillStyle="#FFF",t.fill();let e=Tt(n);return e.keep=!0,e})(),ws=new Set;function dp(n={}){if(qs&&Wa[qs])return i_(Wa[qs],qs,n);let t=we,e=bs[t],i=re[t],s=n.look||yp(),r=Il[s]||Il.classic,a=r.art,o={r:t,C:i,look:s,x:n.x??180,y:n.y??360,z:n.z??0,tags:[],stars:0,s:{rx:0,ry:0,vrx:0,vry:0,fl:n.up?0:Math.PI,vfl:0,rz:0,vrz:0,sc:n.sc??1},flipT:n.up?0:Math.PI,tx:0,ty:0,k:120,d:11,kf:150,df:14,idle:!1,ptr:null,interactive:!1,jit:0,heat:0,glowA:0,glowCol:new Pt(i),sheen:-1,sheenA:0,vis:1},l=new He,c=new He;l.add(c),o.root=l,o.body=c;let h=r.face(t),u=r.maps(t);o.faceCv=h,o.faceT=Tt(h);let d=new Pt("#FFFFFF").lerp(new Pt(i),.45),f=new bn({color:r.edge??e[1],metalness:r.edgeMetal??1,roughness:r.edgeRough??.3,envMapIntensity:1.1}),g=r.bezelCol?new bn({color:r.bezelCol,metalness:1,roughness:.28,envMapIntensity:1.1}):f,x=new Ot(new ns(Xr(new Xe,Ii,ds,Ca),{depth:Vs,bevelEnabled:!1,curveSegments:10}),[new Ve({visible:!1}),f]);x.position.z=-Vs/2,c.add(x),o.front=new Mn({map:o.faceT,roughnessMap:u.orm,metalnessMap:u.orm,metalness:1,roughness:1,envMapIntensity:.9,iridescence:t>=3&&!r.matte?1:0,iridescenceMap:u.foil,iridescenceIOR:1.7,iridescenceThicknessRange:[180,720],clearcoat:r.matte?0:.3,clearcoatRoughness:.3,emissive:d,emissiveIntensity:0,...r.normal==="brushed"?{normalMap:Jy(),normalScale:new j(.35,.35)}:{}});let m=new Ot(La(Ii,ds,Ca),o.front);m.position.z=Vs/2+5e-4,c.add(m);let p=Tt(r.back(t)),T=r.backOrm?r.backOrm():_y();o.backM=new Mn({map:p,roughnessMap:T,metalnessMap:T,metalness:r.matte?0:1,roughness:1,envMapIntensity:.9,clearcoat:r.matte?0:.3,clearcoatRoughness:.3,emissive:d,emissiveIntensity:0});let M=new Ot(La(Ii,ds,Ca),o.backM);M.rotation.y=Math.PI,M.position.z=-Vs/2-5e-4,c.add(M);let y=Vs/2+(a.bezel?a.depth??.045:a.z??.01);if(a.bezel){let P=Yy(a,g);P.position.set(0,a.y,Vs/2),c.add(P)}o.artM=new Mn({map:Xy(a.photo),roughness:.55,metalness:0,clearcoat:r.matte?0:.35,clearcoatRoughness:.32,envMapIntensity:.4,iridescence:t>=5&&!r.matte?.45:0,iridescenceIOR:1.5,iridescenceThicknessRange:[200,500],emissive:d,emissiveIntensity:0});let D=new Ot($y(a),o.artM);if(D.position.set(0,a.y,y),a.rot&&(D.rotation.z=a.rot),c.add(D),r.over){o.overT=Tt(r.over(t)),o.overM=new bn({map:o.overT,transparent:!0,roughness:.5,metalness:0,envMapIntensity:.3,depthWrite:!1,emissive:d,emissiveIntensity:0});let P=new Ot(La(Ii,ds,Ca),o.overM);P.position.z=y+.004,c.add(P)}if(o.sparkU={uMask:{value:u.foil},uAmt:{value:(t>=5?1:t>=3?.7:0)*(r.sparkle??1)},uSheen:{value:-1},uSheenA:{value:0},uDens:{value:1}},t<3){let P=Pe(4,4);P.getContext("2d").fillStyle="#FFF",P.getContext("2d").fillRect(0,0,4,4),o.sparkU.uMask.value=Tt(P,!1)}let S=new Ot(La(Ii,ds,Ca),new Te({uniforms:o.sparkU,vertexShader:by,fragmentShader:My,transparent:!0,blending:Qn,depthWrite:!1}));S.position.z=r.over?y+.006:Vs/2+.002,c.add(S),o.glowM=new Ve({map:up,color:o.glowCol,transparent:!0,opacity:0,blending:Qn,depthWrite:!1});let C=new Ot(new ei(Ii*2.2,ds*1.75),o.glowM);return C.position.z=-.3,l.add(C),o.glow=C,o.mats=[f,o.front,o.backM,o.artM],o.redraw=()=>{let P=r.face(t,o.tags,o.stars);o.faceCv=P,o.faceT.image=P,o.faceT.needsUpdate=!0,r.over&&(o.overT.image=r.over(t,o.tags,o.stars),o.overT.needsUpdate=!0)},Ss.add(l),ws.add(o),o.root.renderOrder=2,o}function Sy(n,t){let e=n.s,i=n.tx,s=n.ty;n.ptr&&n.interactive?(i=n.ptr.y*.42,s=n.ptr.x*.52):n.idle&&(i=.11*Math.sin($e*1.05+1)+.05*Math.sin($e*2.2),s=.22*Math.sin($e*.75)+.07*Math.sin($e*1.7+2));let r=n.free?0:Math.max(1,Math.ceil(t/.01)),a=t/Math.max(1,r);for(let h=0;h<r;h++)e.vrx+=(n.k*(i-e.rx)-n.d*e.vrx)*a,e.rx+=e.vrx*a,e.vry+=(n.k*(s-e.ry)-n.d*e.vry)*a,e.ry+=e.vry*a,e.vfl+=(n.kf*(n.flipT-e.fl)-n.df*e.vfl)*a,e.fl+=e.vfl*a,e.vrz+=(90*((n.rzT||0)-e.rz)-9*e.vrz)*a,e.rz+=e.vrz*a;let o=n.jit?Gt(-n.jit,n.jit)/100:0,l=n.jit?Gt(-n.jit,n.jit)/100:0;n.root.position.set(ly(n.x)+o,cy(n.y)+l,n.z),n.body.rotation.set(e.rx,e.ry+e.fl,e.rz,"ZXY"),n.root.scale.setScalar(e.sc),n.glow.rotation.set(e.rx*.6,e.ry*.6,e.rz);let c=n.heat*n.heat*.75;n.front.emissiveIntensity=c,n.backM.emissiveIntensity=c,n.artM.emissiveIntensity=c,n.overM&&(n.overM.emissiveIntensity=c),n.glowM.opacity=n.glowA*.5,n.glowM.color.copy(n.glowCol),n.sparkU.uSheen.value=n.sheen,n.sparkU.uSheenA.value=n.sheenA,n.root.visible=n.vis>.01,n.tick&&n.tick(t)}function fp(n){n.sheen=-.4,n.sheenA=.55,De(n,{sheen:1.9},1e3,ee.inOut).then(()=>{n.sheenA=0})}function Iu(n,t,e=260,i=15){n.flipT=t,n.kf=e,n.df=i}function wy(n){n.idle=!0,n.interactive=!0,n.k=70,n.d=9,fp(n)}function Al(n,t,e){let i=n.s.sc;return[n.x+(t-100)*i,n.y+(e-142)*i]}async function Ey(n,t){for(let e=0;e<Se[n.r];e++){n.stars=e+1,n.redraw(),await ke(70,t),Z.play("tick",1300+e*200);let[i,s]=n.starXY?Al(n,...n.starXY(e,Se[n.r])):Al(n,184-(Se[n.r]-1-e)*13,21);ys(i,s,n.C,1,1),Dn(i,s,n.C,7,100,{g:0,life:.5}),Ui(i,s,n.C,18,2,.3),n.s.vrx-=1.2,await ke(70,t)}}Sn.addEventListener("pointermove",n=>{let t=Sn.getBoundingClientRect(),e=t.width/360,i=(n.clientX-t.left)/e,s=(n.clientY-t.top)/e;ws.forEach(r=>{if(!r.interactive)return;let a=r.s.sc,o=(i-r.x)/(100*a),l=(s-r.y)/(142*a);r.ptr=Math.abs(o)<1.25&&Math.abs(l)<1.2?{x:Math.max(-1,Math.min(1,o)),y:Math.max(-1,Math.min(1,l))}:null})});Sn.addEventListener("pointerleave",()=>ws.forEach(n=>{n.ptr=null}));Au=gy;window.__warm=(async()=>{if(await new Promise(e=>setTimeout(e,30)),Ks)return;let n=we,t=new He;try{op(5);for(let s of[1,3,5]){we=s;let r=dp({x:180,y:380,up:!0});Ss.remove(r.root),ws.delete(r),t.add(r.root)}we=n;let e=new Vn(.1,.1,.1);[Wn.gold(),Wn.brass(),Wn.steel(),Wn.gunmetal(),Wn.chrome(),Wn.paint("#2D5A61"),Wn.matte("#2D5A61"),Wn.gem("#A7D3CC"),Wn.glass(),Wn.light("#FFFFFF",2),Wn.add("#FFFFFF",1),By("#A7D3CC")].forEach(s=>t.add(new Ot(e,s))),t.add(ky("#FFFFFF",1,1),Ly("#FFFFFF",2,.3,1)),Ne.setRenderTarget(En.readBuffer);let i=Ne.compileAsync(t,qa,fi);Ne.setRenderTarget(null),await i}catch(e){console.warn("warm",e.message)}we=n})();document.fonts.load('600 16px "Google Sans Flex"');var Ul=[];ee.back=n=>1+2.70158*Math.pow(n-1,3)+1.70158*Math.pow(n-1,2);ee.outBack=ee.back;ee.inBack=n=>2.70158*n*n*n-1.70158*n*n;function Ty(n,t,e,i={}){let s=re[e],r=e>=5?1:e>=3?.62:.3;Z.play("boom",r),Z.play("crack"),i.white===!0?Ga(.25+.45*r,.22+.3*r):i.white!==!1&&Ga(.1+.22*r,.2+.2*r),vs({kind:"bloom",x:n,y:t,color:s,a:.75,R:260+180*r,life:.5+.4*r}),Ui(n,t,"#FFFFFF",110+120*r,4,.42),Ui(n,t,s,160+200*r,9,.7,.06),Dn(n,t,s,Math.round(30+120*r),360+460*r,{g:280,bk:!0}),Dn(n,t,"#FFFFFF",Math.round(12+30*r),700,{g:0,life:.35,drag:3,bk:!0}),vs({kind:"flare",x:n,y:t,color:s,len:150+160*r,th:7+6*r,core:90+60*r,a:1,life:.5+.5*r}),ys(n,t,s,Math.round(4+10*r),130,{delay:.3,bk:!0}),Ys(.3+.55*r),_l(55+75*r)}async function Ay(n,t){let e=re[we],i=Ge[we],s=Pe(48,48);ci(s.getContext("2d"),24,24,44,Fi[Tl[we]]);let r=iy(`<div style="position:absolute;left:0;right:0;top:${n}px;display:flex;justify-content:center;align-items:center;gap:8px;color:${e};font-size:26px;font-weight:700;font-variant-numeric:tabular-nums;text-shadow:0 1px 2px rgba(4,14,16,.85), 0 0 18px ${Qt(e,.6)};opacity:0"><img alt="" src="${s.toDataURL()}" width="24" height="24"><span>+0</span></div>`);It(r,[{opacity:0,transform:"translateY(10px) scale(.9)"},{opacity:1,transform:"none"}],220,{ease:"cubic-bezier(.3,1.4,.5,1)"});let a=r.querySelector("span");for(let o=0;o<=i;o+=Math.ceil(i/8))a.textContent="+"+o,Z.play("tick",1800),await ke(32,t);return a.textContent="+"+i,It(r,[{transform:"scale(1.18)"},{transform:"scale(1)"}],200),r}async function Cy(n,t,e={}){await ke(e.delay??400,n),Z.play("chime",t.r),Z.play("shimmer"),wy(t);let i=ty(40,320,t.y+40,Math.min(740,t.y+340),[t.C,"#FFFFFF"],t.r>=5?20:10,60);await Ey(t,n);let s=await Ay(e.pointsY??t.y+162*t.s.sc,n);await ke(t.r>=5?700:450,n),Bu(s,200),await Py(n,t),i.dead=!0}var dn=[{label:"Cameras",col:"#4F8A5B",n:41},{label:"Film",col:"#3E8DB5",n:23},{label:"To buy",col:"#B5705A",n:12},{label:"Gifts",col:"#8A6FB5",n:7}],Fu=["film camera","1970s","rangefinder"],fs=()=>({space:dn[0],tags:Fu.slice(),note:""}),hi=n=>[...n.space?[{k:"space",label:n.space.label,col:n.space.col}]:[],...n.tags.map(t=>({k:"tag",label:t}))],Cl=n=>n.replace(/[<>&"]/g,"");function $r(n){n.tags=hi(n.ts),n.redraw();let[t,e]=Al(n,70,261);ys(t,e,n.C,2,40)}var pp='<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>';function mp(n,t){let e=()=>{if(n.scrollHeight>n.clientHeight+1)return!0;let s=n.getBoundingClientRect();for(let r of t.querySelectorAll("button, span, label, textarea, input")){let a=r.getBoundingClientRect();if(a.width&&(a.right>s.right+1||a.left<s.left-1))return!0}return!1},i=()=>{t.style.zoom="";let s=1;for(let r=0;r<12&&s>.66&&e();r++)s-=.03,t.style.zoom=s};i(),new MutationObserver(i).observe(t,{childList:!0,subtree:!0})}function Ua(n,t,e={}){let i=We('<div class="ted"></div>'),s=!1,r=()=>{if(e.tpl){i.innerHTML=e.tpl(n,s),e.after?.(i,n);return}i.innerHTML=`${e.space===!1?"":`<div class="lbl">Space</div><div class="row"><button type="button" class="spc"><i style="background:${n.ts.space.col}"></i>${n.ts.space.label}${pp}</button></div>
      <div class="row pick"${s?"":" hidden"}>${dn.filter(o=>o!==n.ts.space).map(o=>`<button type="button" class="pk" data-s="${o.label}"><i style="background:${o.col}"></i>${o.label}</button>`).join("")}</div>`}
      <div class="lbl">${e.tagLabel||"Tags"}</div><div class="row">${n.ts.tags.map(o=>`<span class="tg"># ${o}<button type="button" class="x" data-t="${o}" aria-label="Remove ${o}">\xD7</button></span>`).join("")}<button type="button" class="add">+ Add tag</button></div>
      ${e.note?`<div class="lbl">Note</div><textarea class="field note" rows="${e.noteRows||3}" placeholder="Why you kept it">${n.ts.note}</textarea>`:""}`};r();let a=()=>{t.hold(),$r(n),Z.play("tick",1500),n.s.vrx-=.5};return i.addEventListener("pointerdown",()=>t.hold()),i.addEventListener("click",o=>{o.stopPropagation();let l=o.target.closest("button");if(l){if(l.classList.contains("spc")){s=!s,r(),t.hold();return}if(l.classList.contains("pk")){n.ts.space=dn.find(c=>c.label===l.dataset.s),s=!1,r(),a(),e.fx?.space?.(i,n.ts.space);return}if(l.classList.contains("x")&&e.fx?.remove){let c=l.closest(".tg");e.fx.remove(c,l.dataset.t).then(()=>{n.ts.tags=n.ts.tags.filter(h=>h!==l.dataset.t),r(),a()},()=>{});return}if(l.classList.contains("x")){let c=l.closest(".tg");It(c,[{opacity:1,transform:"none"},{opacity:0,transform:"scale(.6)"}],140).finished.then(()=>{n.ts.tags=n.ts.tags.filter(h=>h!==l.dataset.t),r(),a()},()=>{});return}if(l.classList.contains("add")){let c=We('<input class="field" placeholder="New tag" aria-label="New tag" style="width:150px">');l.replaceWith(c),c.focus(),t.hold(),c.addEventListener("keydown",h=>{if(h.key==="Enter"&&c.value.trim()){let u=Cl(c.value.trim()).slice(0,24);n.ts.tags.push(u),r(),a(),e.fx?.add?.(i,u)}}),c.addEventListener("blur",()=>{c.isConnected&&r()})}}}),i.addEventListener("input",o=>{o.target.classList.contains("note")&&(n.ts.note=o.target.value)}),i}function Ry(n){let t=We('<button type="button" class="done"><i class="left"></i><span style="position:relative">Done</span></button>');return t.addEventListener("click",e=>{e.stopPropagation(),n.close()}),t}function Na(n,t){return new Promise((e,i)=>{let s=!0,r=!1,a=0,o=!1,l=5e3*Bi,h=t({hold(){r||(r=!0,h.onHold&&h.onHold())},close:()=>u(),start(){o=!0}});h.waitStart||(o=!0);async function u(){if(s){s=!1,bl=null,Z.play("tick",900);try{await h.leave()}catch{}h.cleanup&&h.cleanup(),n===Tn?e():i(new Ms)}}bl=u,Ll(d=>{if(!s)return!1;if(n!==Tn)return s=!1,h.cleanup&&h.cleanup(),i(new Ms),!1;if(!r&&o&&(a+=d,h.tick&&h.tick(Math.min(1,a/l)),a>=l))return u(),!1})})}var vu="border-radius:26px;background:rgba(11,34,39,.97);box-shadow:0 0 0 1px #1E474E, 0 18px 40px rgba(0,0,0,.45);padding:14px;pointer-events:auto",Xa=[{name:"Panel",d:"The card moves up and a sheet rises under it with the space, the tags and Add. Done runs down with the time left.",run(n,t){return Na(n,e=>{De(t,{y:290},320,ee.inOut),De(t.s,{sc:.84},320,ee.inOut);let i=di(`<div class="tagp" style="position:absolute;left:10px;right:10px;bottom:10px;${vu}"></div>`);i.appendChild(Ua(t,e));let s=Ry(e);s.style.marginTop="14px",i.appendChild(s),Z.play("pop"),It(i,[{opacity:0,transform:"translateY(60px)"},{opacity:1,transform:"none"}],320,{ease:"cubic-bezier(.2,.9,.3,1.05)"});let r=s.querySelector(".left");return{tick:a=>{r.style.width=100-a*100+"%"},onHold:()=>{r.style.opacity="0"},leave:()=>It(i,[{opacity:1},{opacity:0,transform:"translateY(40px)"}],200,{ease:"ease-in"}).finished,cleanup:()=>i.remove()}})}},{name:"Auto-loot",d:"The suggested space and tags pop up round the card like pickups and fly onto it by themselves, one by one. A small bar under the card then lets you drop one or add your own.",run(n,t){return Na(n,e=>{let i=t.ts;t.ts={space:i.space,tags:[],note:""},t.tags=[],t.redraw(),De(t,{y:320},300,ee.inOut),De(t.s,{sc:.92},300,ee.inOut);let s=di('<div class="tagp" style="position:absolute;inset:0;pointer-events:none"></div>'),r=[{k:"space",label:i.space.label,col:i.space.col},...i.tags.map(h=>({k:"tag",label:h}))],a=[[-112,116],[96,150],[-96,512],[104,548]],o=r.map((h,u)=>{let d=We(`<div style="position:absolute;left:${180+a[u][0]}px;top:${a[u][1]}px;transform:translateX(-50%);height:34px;padding:0 13px;border-radius:17px;display:flex;align-items:center;gap:6px;font-size:14px;font-weight:${h.k==="space"?600:500};white-space:nowrap;${h.k==="space"?"background:#F4F2EF;color:#0E333A":"background:rgba(11,34,39,.92);color:#E8ECEB;box-shadow:inset 0 0 0 1px "+Qt(t.C,.7)};box-shadow:0 0 22px ${Qt(t.C,.55)}${h.k==="space"?"":", inset 0 0 0 1px "+Qt(t.C,.7)};opacity:0">${h.k==="space"?`<i style="width:9px;height:9px;border-radius:50%;background:${h.col}"></i>`:"#"} ${h.label}</div>`);return s.appendChild(d),It(d,[{opacity:0,transform:"translateX(-50%) scale(.3)"},{opacity:1,transform:"translateX(-50%) scale(1.08)",offset:.7},{opacity:1,transform:"translateX(-50%) scale(1)"}],300,{delay:u*90,ease:"cubic-bezier(.3,1.4,.5,1)"}),It(d,[{translate:"0 -3px"},{translate:"0 3px"}],900+u*130,{iter:1/0,dir:"alternate",ease:"ease-in-out"}),d});Z.play("shimmer");let l=null,c=null;return(async()=>{try{await ke(650,n);for(let u=0;u<o.length;u++){let d=o[u],f=r[u],g=d.getBoundingClientRect(),x=Sn.getBoundingClientRect().width/360,m=Sn.getBoundingClientRect(),p=(g.left+g.width/2-m.left)/x,T=(g.top+g.height/2-m.top)/x,[M,y]=Al(t,40+u*30,261);d.getAnimations().forEach(D=>D.cancel()),d.style.opacity="1",Z.play("swoop"),await It(d,[{transform:"translateX(-50%)",opacity:1},{transform:`translate(calc(-50% + ${M-p}px), ${y-T}px) scale(.35)`,opacity:.2}],280,{ease:"cubic-bezier(.6,0,.4,1)"}).finished.catch(()=>{}),Ni(n),d.remove(),f.k==="space"?t.ts.space=i.space:t.ts.tags.push(f.label),t.tags=hi(t.ts),t.redraw(),Z.play("ting",Math.min(2,u)),Dn(M,y,t.C,12,140,{g:0,life:.4}),Ui(M,y,t.C,34,2,.35),t.s.vrx-=.6,t.s.vrz+=(u%2?1:-1)*.05,await ke(110,n)}l=We(`<div style="position:absolute;left:10px;right:10px;bottom:10px;${vu};padding:12px"><div style="display:flex;justify-content:space-between;align-items:center;margin:0 2px 8px"><span style="font-size:13px;color:#CFE7E2">Added for you</span><span style="font-size:12px;color:#8FA9AC">tap \xD7 to drop one</span></div></div>`),l.appendChild(Ua(t,e,{space:!0}));let h=We(`<div style="position:absolute;left:24px;right:24px;bottom:5px;height:2px;border-radius:1px;background:rgba(255,255,255,.08)"><i style="position:absolute;left:0;top:0;bottom:0;width:100%;border-radius:1px;background:${t.C}"></i></div>`);l.appendChild(h),c=h.firstElementChild,s.appendChild(l),It(l,[{opacity:0,transform:"translateY(40px)"},{opacity:1,transform:"none"}],300,{ease:"cubic-bezier(.2,.9,.3,1.05)"}),e.start()}catch{}})(),{waitStart:!0,tick:h=>{c&&(c.style.width=100-h*100+"%")},onHold:()=>{c&&(c.style.opacity="0")},leave:async()=>{l||(t.ts=i,$r(t)),l&&await It(l,[{opacity:1},{opacity:0,transform:"translateY(30px)"}],200).finished},cleanup:()=>s.remove()}})}},{name:"Card back",key:"C",d:"The card turns over and grows to fill the screen, and its back is where you edit: the space, the tags, and a proper note field. Then it turns face up again and goes.",run(n,t){return Na(n,e=>{t.interactive=!1,t.idle=!1,t.tx=0,t.ty=0,t.k=120,t.d=18,t.ptr=null,Iu(t,t.flipT+Math.PI,110,17),De(t,{y:352},460,ee.inOut),De(t.s,{sc:1.45},460,ee.inOut),Z.play("whoosh");let i=di(`<div class="tagp" style="position:absolute;left:46px;top:157px;width:268px;height:390px;border-radius:12px;background:rgba(5,16,18,.9);padding:14px;pointer-events:auto;display:flex;flex-direction:column;opacity:0">
        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:4px"><span style="font-size:15px;font-weight:600;color:${t.C}">On the back</span><span style="font-size:12px;color:#8FA9AC">Rangefinder camera</span></div></div>`),s=Ua(t,e,{note:!0,noteRows:3});i.appendChild(s);let r=We(`<div style="margin-top:auto;height:3px;border-radius:2px;background:rgba(255,255,255,.08);position:relative"><i style="position:absolute;left:0;top:0;bottom:0;width:100%;border-radius:2px;background:${t.C}"></i></div>`);i.appendChild(r),mp(i,s),It(i,[{opacity:0,transform:"scale(.97)"},{opacity:1,transform:"none"}],220,{delay:430});let a=r.firstElementChild;return{tick:o=>{a.style.width=100-o*100+"%"},onHold:()=>{a.style.opacity="0"},leave:async()=>{It(i,[{opacity:1},{opacity:0}],140),await ke(120,n),Iu(t,t.flipT+Math.PI,110,17),De(t,{y:360},460,ee.inOut),De(t.s,{sc:1},460,ee.inOut),Z.play("whoosh"),await ke(520,n),t.k=70,t.d=9,t.idle=!0,t.interactive=!0},cleanup:()=>i.remove()}})}},{name:"Chests",d:"Your spaces wait along the bottom as chests. The suggested one is already open and glowing; tap another to move the find there. Tags sit above, with \xD7 and Add.",run(n,t){return Na(n,e=>{De(t,{y:270},320,ee.inOut),De(t.s,{sc:.78},320,ee.inOut);let i=c=>`<button type="button" class="ch" data-s="${c.label}" style="position:relative;flex:1;height:104px;border:0;border-radius:16px;background:transparent;cursor:pointer;color:#E8ECEB;padding:0">
        <span class="sug" style="position:absolute;left:0;right:0;top:-2px;font-size:10.5px;color:${t.C};opacity:0">Suggested</span>
        <span class="lid" style="position:absolute;left:14px;right:14px;top:16px;height:18px;border-radius:9px 9px 3px 3px;background:linear-gradient(180deg, ${Qt(c.col,1)}, ${Qt(c.col,.65)});box-shadow:inset 0 1px 0 rgba(255,255,255,.4);transform-origin:50% 100%;transition:transform .25s cubic-bezier(.3,1.4,.5,1)"></span>
        <span class="glw" style="position:absolute;left:18px;right:18px;top:30px;height:8px;border-radius:50%;background:${c.col};filter:blur(6px);opacity:0;transition:opacity .25s"></span>
        <span style="position:absolute;left:14px;right:14px;top:35px;height:30px;border-radius:3px 3px 9px 9px;background:linear-gradient(180deg, #1C4B52, #0B2227);box-shadow:inset 0 0 0 1.5px ${Qt(c.col,.75)}"><i style="position:absolute;left:50%;top:6px;width:9px;height:11px;margin-left:-4.5px;border-radius:3px;background:${c.col}"></i></span>
        <span style="position:absolute;left:0;right:0;top:71px;font-size:12.5px;text-align:center">${c.label}</span><span style="position:absolute;left:0;right:0;top:87px;font-size:10px;color:#8FA9AC;text-align:center">${c.n} kept</span></button>`,s=di(`<div class="tagp" style="position:absolute;left:10px;right:10px;bottom:10px;${vu};padding:12px"></div>`);s.appendChild(Ua(t,e,{space:!1}));let r=We(`<div style="display:flex;gap:2px;margin-top:12px;border-top:1px solid #1E474E;padding-top:10px">${dn.map(i).join("")}</div>`);s.appendChild(r);let a=We(`<div style="position:absolute;left:24px;right:24px;bottom:5px;height:2px;border-radius:1px;background:rgba(255,255,255,.08)"><i style="position:absolute;left:0;top:0;bottom:0;width:100%;border-radius:1px;background:${t.C}"></i></div>`);s.appendChild(a);let o=()=>r.querySelectorAll(".ch").forEach(c=>{let h=c.dataset.s===t.ts.space.label;c.querySelector(".lid").style.transform=h?"translateY(-9px) rotate(-22deg)":"none",c.querySelector(".glw").style.opacity=h?"1":"0",c.querySelector(".sug").style.opacity=h&&t.ts.space===dn[0]?"1":"0",c.style.background=h?"rgba(244,242,239,.06)":"transparent"});o(),r.addEventListener("click",c=>{c.stopPropagation();let h=c.target.closest(".ch");h&&(e.hold(),t.ts.space=dn.find(u=>u.label===h.dataset.s),o(),$r(t),Z.play("clunk",.3))}),Z.play("pop"),It(s,[{opacity:0,transform:"translateY(60px)"},{opacity:1,transform:"none"}],320,{ease:"cubic-bezier(.2,.9,.3,1.05)"});let l=a.firstElementChild;return{tick:c=>{l.style.width=100-c*100+"%"},onHold:()=>{l.style.opacity="0"},leave:()=>It(s,[{opacity:1},{opacity:0,transform:"translateY(40px)"}],200,{ease:"ease-in"}).finished,cleanup:()=>s.remove()}})}}],gp=2;async function Py(n,t){kl.querySelectorAll(".skiph").forEach(e=>e.remove()),t.ts=t.ts||fs(),$r(t),await Xa[gp].run(n,t),await Fy(n,t)}function Iy(n){return di(`<div style="position:absolute;right:14px;top:34px;height:48px;display:flex;align-items:center;gap:0;opacity:0">
    <div class="lab" style="height:32px;margin-right:-14px;padding:0 22px 0 12px;border-radius:16px 0 0 16px;background:rgba(11,34,39,.92);color:#F4F2EF;font-size:12.5px;display:flex;align-items:center;gap:6px;white-space:nowrap;clip-path:inset(0 0 0 100%)"></div>
    <div class="bg" style="position:relative;width:48px;height:48px;border-radius:16px;background:radial-gradient(120% 100% at 30% 20%, #1E555D, ${Lu});display:grid;place-items:center;box-shadow:0 0 0 2px ${n}, 0 0 20px ${Qt(n,.45)}, 0 8px 20px rgba(0,0,0,.3)">
    <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="#F4F2EF" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round"><path d="M9.5 4.5V3.8a2.5 2.5 0 0 1 5 0v.7"/><path d="M6 9.5a4.5 4.5 0 0 1 4.5-4.5h3A4.5 4.5 0 0 1 18 9.5V19a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2z"/><path d="M6 11.5h12"/><path d="M8.5 15h7v3.5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1z"/><path d="M12 11.5v1.6"/></svg>
    <span class="cnt" style="position:absolute;bottom:-9px;left:50%;transform:translateX(-50%);height:18px;padding:0 6px;border-radius:9px;background:${Yv};color:#0E333A;font-size:10.5px;font-weight:700;display:flex;align-items:center;white-space:nowrap;font-variant-numeric:tabular-nums${Gr.before==null?";display:none":""}">${Sp(!1)}</span></div></div>`)}async function xp(n,t,e,i){let s=Iy(e),r=s.querySelector(".bg");It(s,[{opacity:0,transform:"translateY(-10px) scale(.8)"},{opacity:1,transform:"none"}],240,{ease:"cubic-bezier(.3,1.5,.5,1)"});let a=Sn.getBoundingClientRect(),o=a.width/360,l=t.getBoundingClientRect(),c=r.getBoundingClientRect(),h=(c.left+c.width/2-l.left-l.width/2)/o,u=(c.top+c.height/2-l.top-l.height/2)/o,d=t.style.transform||"";await It(t,[{transform:d},{transform:`translateY(-14px) scale(1.05) ${d}`}],150,{ease:"cubic-bezier(.3,0,.3,1)"}).finished.catch(()=>{}),Ni(n),Z.play("swoop");let f=vs({kind:"comet",follow:t,color:e,w:3,trail:[],n:16});await It(t,[{transform:`translateY(-14px) scale(1.05) ${d}`,opacity:1},{transform:`translate(${h*.35}px, ${u*.2-50}px) scale(.55) rotate(8deg)`,opacity:1,offset:.45},{transform:`translate(${h}px, ${u}px) scale(.1) rotate(20deg)`,opacity:.5}],520,{ease:"cubic-bezier(.45,.05,.55,.95)"}).finished.catch(()=>{}),Ni(n),t.remove(),f.dead=!0,Z.play("pop"),Z.play("shimmer");let[g,x]=Ba(r);Dn(g,x,e,22,180,{g:0,life:.5}),Ui(g,x,e,46,3,.4),ys(g,x,e,4,22),It(r,[{transform:"scale(1)"},{transform:"scale(1.22)"},{transform:"scale(1)"}],280,{ease:"ease-out"}),s.querySelector(".cnt").textContent=Sp(!0);let m=s.querySelector(".lab");m.innerHTML=`${i?`<i style="width:8px;height:8px;border-radius:50%;background:${i.col}"></i>${Cl(i.label)}`:""}<span style="color:${e};font-weight:600">+${Ge[we]}</span>`,It(m,[{clipPath:"inset(0 0 0 100%)"},{clipPath:"inset(0 0 0 0)"}],260,{ease:"cubic-bezier(.2,.8,.2,1)",delay:80}),await ke(1500,n),await It(s,[{opacity:1},{opacity:0,transform:"translateY(-8px)"}],240).finished.catch(()=>{})}async function Fy(n,t){let e=t.find?Mp(t.key,t.tags,t.stars):null,i=t.s.sc,s=(e?e.w*t.fit:200)*i,r=(e?e.h*t.fit:284)*i;Pu(1,1);let a=di(`<img alt="" src="${e?e.src:_p(t.look,t.tags,t.stars)}" style="position:absolute;left:${t.x-s/2}px;top:${t.y-r/2}px;width:${s}px;height:${r}px;${e?bp(e,t.C):`border-radius:${16*i}px;box-shadow:0 0 22px ${Qt(t.C,.5)}, 0 10px 24px rgba(0,0,0,.35)`};transform:perspective(900px) rotateX(${(-t.s.rx).toFixed(3)}rad) rotateY(${((t.s.ry+t.s.fl)%(Math.PI*2)).toFixed(3)}rad)">`);await a.decode?.().catch(()=>{}),$a(160),await xp(n,a,t.C,t.ts.space)}function $a(n=160){Ks=!1,bl=null,kl.querySelectorAll(".skiph, .tagp").forEach(s=>s.remove()),hp(),Va=[],xs=0,Ha.forEach(s=>jv(s,n));let t=[...Ml.children,...Nu.children];t.forEach(s=>{s.style.pointerEvents="none",Bu(s,n)}),Zs(()=>t.forEach(s=>s.remove()),n*Bi+40),wn.getAnimations().forEach(s=>s.cancel()),wn.style.transform="";let e=getComputedStyle(ka).filter,i=getComputedStyle(ka).transform;ka.getAnimations().forEach(s=>s.cancel()),It(ka,[{filter:e,transform:i},{filter:"none",transform:"none"}],n+80)}function Dy(){let n=di('<div class="skiph" style="position:absolute;left:0;right:0;bottom:22px;display:flex;justify-content:center;opacity:0"><span style="display:flex;align-items:center;gap:6px;height:28px;padding:0 12px;border-radius:14px;background:rgba(5,18,21,.5);color:rgba(244,242,239,.8);font-size:12.5px">Tap to skip<svg width="14" height="14" viewBox="0 0 24 24" fill="rgba(244,242,239,.8)"><path d="M4 6l8 6-8 6zM12 6l8 6-8 6z"/></svg></span></div>');It(n,[{opacity:0},{opacity:1}],300,{delay:700})}async function Nl(){let n=++Tn;Uu=1,za=1,Cu(),Li.length=0;let t=document.getElementById("share");t.getAnimations().forEach(c=>c.cancel()),t.style.transform="translateY(230px)",$a(150);let e=re[we],i=fs(),s=qs&&Wa[qs],r=s?Mp(qs,hi(i),Se[we]):null,a=r?r.w*ku(s)*.75:150,o=r?r.h*ku(s)*.75:213,l=di(`<img alt="" src="${r?r.src:_p(yp(),hi(i),Se[we])}" style="position:absolute;left:${180-a/2}px;top:${379.5-o/2}px;width:${a}px;height:${o}px;${r?bp(r,e):`border-radius:12px;box-shadow:0 0 26px ${Qt(e,.6)}, 0 12px 26px rgba(0,0,0,.35)`};opacity:0">`);try{await l.decode?.().catch(()=>{}),Z.play("whoosh"),await It(l,[{opacity:0,transform:"scale(.4) rotate(-8deg)"},{opacity:1,transform:"scale(1.08) rotate(2deg)",offset:.7},{opacity:1,transform:"none"}],300,{ease:"cubic-bezier(.2,.9,.3,1)"}).finished,Ni(n),l.style.opacity="1",Ui(180,380,e,110,4,.45),Dn(180,380,e,26,240,{g:120}),Z.play("chime",we),await ke(260,n),await xp(n,l,e,i.space)}catch(c){c instanceof Ms||console.error(c)}n===Tn&&Fl()}function vp(n){let t=0,e=i=>{t+=i,n(i,t)};return Oi.tick.add(e),()=>Oi.tick.delete(e)}var Rl=(n,t=1)=>new Pt(n).multiplyScalar(t),Ou={blending:Yh,blendEquation:Si,blendSrc:ua,blendDst:il,blendSrcAlpha:Zh,blendDstAlpha:il},Wn={gold:(n={})=>new Mn({color:"#F0B952",metalness:1,roughness:.2,envMapIntensity:1.5,clearcoat:.25,...n}),brass:(n={})=>new bn({color:"#C9A56A",metalness:1,roughness:.32,envMapIntensity:1.3,...n}),steel:(n={})=>new bn({color:"#C8D3D2",metalness:1,roughness:.24,envMapIntensity:1.4,...n}),gunmetal:(n={})=>new bn({color:"#4A5859",metalness:.9,roughness:.38,envMapIntensity:1.2,...n}),chrome:(n={})=>new bn({color:"#FFFFFF",metalness:1,roughness:.05,envMapIntensity:1.6,...n}),paint:(n,t={})=>new Mn({color:n,metalness:.1,roughness:.42,clearcoat:.8,clearcoatRoughness:.2,envMapIntensity:1,...t}),matte:(n,t={})=>new bn({color:n,metalness:0,roughness:.85,envMapIntensity:.6,...t}),gem:(n,t={})=>new Mn({color:"#FFFFFF",metalness:0,roughness:0,transmission:1,thickness:1.4,ior:2.1,dispersion:5,attenuationColor:new Pt(n),attenuationDistance:.7,specularIntensity:1,envMapIntensity:2.2,flatShading:!0,...t}),glass:(n={})=>new Mn({color:"#FFFFFF",metalness:0,roughness:.04,transmission:1,thickness:.3,ior:1.5,envMapIntensity:1.6,transparent:!0,...n}),light:(n,t=4,e={})=>new Ve({color:Rl(n,t),toneMapped:!1,...e}),add:(n,t=1,e={})=>new Ve({color:Rl(n,t),transparent:!0,...Ou,depthWrite:!1,toneMapped:!1,...e})},pl=null;function ky(n,t=1,e=2){if(!pl){let s=Pe(128,128),r=s.getContext("2d"),a=r.createRadialGradient(64,64,0,64,64,64);a.addColorStop(0,"rgba(255,255,255,1)"),a.addColorStop(.2,"rgba(255,255,255,.55)"),a.addColorStop(.5,"rgba(255,255,255,.14)"),a.addColorStop(1,"rgba(255,255,255,0)"),r.fillStyle=a,r.fillRect(0,0,128,128),pl=Tt(s),pl.keep=!0}let i=new Go(new ga({map:pl,color:Rl(n,e),transparent:!0,...Ou,depthWrite:!1,toneMapped:!1}));return i.scale.setScalar(t),i}function Ly(n,t=6,e=.5,i=3){let s=new Te({uniforms:{uC:{value:Rl(n,i)},uA:{value:1},uT:{value:0}},transparent:!0,...Ou,depthWrite:!1,side:vn,toneMapped:!1,vertexShader:"varying vec2 vUv; varying vec3 vN; varying vec3 vV; void main(){ vUv = uv; vec4 wp = modelMatrix * vec4(position,1.); vN = normalize(mat3(modelMatrix) * normal); vV = normalize(cameraPosition - wp.xyz); gl_Position = projectionMatrix * viewMatrix * wp; }",fragmentShader:"uniform vec3 uC; uniform float uA, uT; varying vec2 vUv; varying vec3 vN; varying vec3 vV; void main(){ float rim = pow(1. - abs(dot(vN, vV)), 1.6); float core = 1. - rim; float fall = pow(1. - vUv.y, 1.8); float flick = .85 + .15 * sin(vUv.y * 30. - uT * 9.); gl_FragColor = vec4(uC * (core * .9 + .25) * fall * flick * uA, 1.); }"}),r=new Ot(new ni(e*.8,e,t,32,1,!0),s);return r.geometry.translate(0,t/2,0),vp((a,o)=>{s.uniforms.uT.value=o}),r.u=s.uniforms,r}var Uy="float h21(vec2 p){ p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); } float vn(vec2 p){ vec2 i = floor(p), f = fract(p); f = f * f * (3. - 2. * f); return mix(mix(h21(i), h21(i + vec2(1, 0)), f.x), mix(h21(i + vec2(0, 1)), h21(i + vec2(1, 1)), f.x), f.y); } float fbm(vec2 p){ float s = 0., a = .5; for (int i = 0; i < 5; i++) { s += a * vn(p); p = p * 2.03 + 1.7; a *= .5; } return s; }";function Ny(n,t=2){let e=n.width,i=n.height,s=n.getContext("2d").getImageData(0,0,e,i).data,r=Pe(e,i),a=r.getContext("2d"),o=a.createImageData(e,i),l=(c,h)=>s[(Math.min(i-1,Math.max(0,h))*e+Math.min(e-1,Math.max(0,c)))*4]/255;for(let c=0;c<i;c++)for(let h=0;h<e;h++){let u=(l(h+1,c)-l(h-1,c))*t,d=(l(h,c+1)-l(h,c-1))*t,f=Math.hypot(u,d,1),g=(c*e+h)*4;o.data[g]=(-u/f*.5+.5)*255,o.data[g+1]=(d/f*.5+.5)*255,o.data[g+2]=(1/f*.5+.5)*255,o.data[g+3]=255}return a.putImageData(o,0,0),Tt(r,!1)}var By=n=>new Mn({color:n,vertexColors:!0,metalness:.55,roughness:.05,envMapIntensity:3,iridescence:.9,iridescenceIOR:1.8,iridescenceThicknessRange:[200,800],clearcoat:1,clearcoatRoughness:.02,emissive:n,emissiveIntensity:.25,flatShading:!0});function Oy(n,t,e={}){let i=dp({x:n,y:t,z:e.z??0,up:e.up??!1,sc:e.sc??1});return i.vis=e.vis??1,i}var In=`uniform float uT, uA, uK, uSat, uBr; uniform vec3 uC; varying vec2 vUv; ${Uy}
vec2 scr(){ return vec2((vUv.x - .5) / .64 + .5, (vUv.y - .5) / .665 + .5); }
float stars(vec2 q, float n, float t){ vec2 g = q * vec2(n * .474, n), id = floor(g); float h = h21(id); vec2 f = fract(g) - .5 - (vec2(h, h21(id + 3.1)) - .5) * .7;
  return smoothstep(.09, 0., length(f)) * step(.84, h) * (.55 + .45 * sin(t * (1. + h * 2.) + h * 50.)) * (h - .84) * 6.; }
vec3 vor(vec2 p){ vec2 n = floor(p), f = fract(p); float d1 = 8., d2 = 8., id = 0.;
  for (int j = -1; j <= 1; j++) for (int i = -1; i <= 1; i++) { vec2 g = vec2(float(i), float(j)), o = vec2(h21(n + g), h21(n + g + 19.7)); vec2 r = g + o - f; float d = dot(r, r);
    if (d < d1) { d2 = d1; d1 = d; id = h21(n + g + 7.7); } else if (d < d2) d2 = d; }
  return vec3(sqrt(d1), sqrt(d2), id); }
float glowAt(vec2 q){ vec2 d = (q - vec2(.5, .51)) * vec2(.474, 1.); return exp(-dot(d, d) * 14.); }
`,Pi="col += uC * glowAt(q) * max(uK - .3, 0.) * .22; col = mix(vec3(dot(col, vec3(.3, .55, .15))), col, uSat) * uBr; gl_FragColor = vec4(col * uA, uA); }",ml="col *= .72 / max(.72, max(col.r, max(col.g, col.b))); "+Pi.replace("* uBr;","* uBr; col *= .72 / max(.72, max(col.r, max(col.g, col.b)));"),zy={aurora:{name:"Aurora",c:"#4FE3A2",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 col = mix(vec3(.012, .02, .05), vec3(.02, .035, .1), q.y) + vec3(.0, .03, .05) * smoothstep(.55, .2, q.y);
    col += vec3(.8, .86, 1.) * (stars(q, 150., uT) + stars(q + 3.3, 70., uT * .7)) * smoothstep(.3, .65, q.y);
    float x = p.x * 3., e = .75 + .35 * uK; vec3 acc = uC;
    for (int i = 0; i < 3; i++) { float fi = float(i);
      float yc = .44 + .09 * fi + .05 * sin(x * 2.1 + uT * .22 + fi * 1.7) + .06 * (fbm(vec2(x * 1.2 + fi * 3., uT * .07)) - .5);
      float dy = q.y - yc, cur = smoothstep(-.012, .01, dy) * exp(-max(dy, 0.) * (7. - fi * 1.6));
      float rays = .45 + .55 * vn(vec2(x * 16. + fi * 5. + uT * .3, uT * .5)) * vn(vec2(x * 5. - uT * .1, fi));
      vec3 ac = mix(vec3(.12, 1., .5), vec3(.6, .32, 1.), clamp(fi * .4 + dy * 5., 0., 1.)); ac = mix(ac, acc, clamp(uK - .4, 0., 1.) * .5);
      col += ac * cur * rays * (.7 - fi * .16) * e + ac * exp(-abs(dy) * 30.) * .12 * e; }
    float r1 = .25 + .07 * fbm(vec2(p.x * 5. + 2., 1.)) + .05 * (1. - abs(p.x * 2.)), r2 = .17 + .06 * fbm(vec2(p.x * 8., 4.)) - .03 * p.x;
    vec3 far = vec3(.04, .07, .11) + vec3(.06, .16, .14) * smoothstep(r1 - .05, r1, q.y) * .6, near = vec3(.015, .025, .04);
    col = mix(col, far, step(q.y, r1)); col = mix(col, near + vec3(.05, .12, .1) * smoothstep(r2 - .03, r2, q.y) * .5, step(q.y, r2));
    col += vec3(.06, .14, .12) * exp(-abs(q.y - r1) * 40.) * .4 * e; ${Pi}`},dawn:{name:"Golden hour",c:"#FFB45C",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 col = mix(vec3(1., .62, .36), vec3(.98, .8, .58), smoothstep(.3, .62, q.y)); col = mix(col, vec3(.42, .6, .86), smoothstep(.6, 1., q.y));
    vec2 sp = (q - vec2(.5, .36)) * vec2(.474, 1.); float sd = length(sp);
    col += vec3(1., .85, .6) * (smoothstep(.045, .04, sd) * 1.2 + exp(-sd * 9.) * .55 + exp(-sd * 3.) * .2) * (.8 + .2 * uK);
    float hi = fbm(vec2(p.x * 3. + uT * .02, q.y * 14.)); col = mix(col, vec3(1., .92, .82), smoothstep(.55, .75, hi) * smoothstep(.62, .8, q.y) * .5);
    for (int i = 0; i < 3; i++) { float fi = float(i), top = .34 - fi * .09;
      float n = fbm(vec2(p.x * (3. + fi) + uT * (.03 + fi * .01) + fi * 7., fi)); float h = top + n * .08;
      float b = fbm(vec2(p.x * 9. + fi * 3. + uT * .02, q.y * 9. + fi)); float m = smoothstep(h + .01, h - .03, q.y + (b - .5) * .03);
      vec3 cc = mix(vec3(1., .78, .6), vec3(.95, .55, .45), fi * .35) * (1. - fi * .12); float rim = exp(-(h - q.y) * 30.) * (1. - fi * .3);
      col = mix(col, cc * (.7 + .3 * b) + vec3(1., .85, .6) * rim * .5, m); }
    ${ml}`},lagoon:{name:"Lagoon",c:"#5FE0D2",frag:`${In}
    float cau(vec2 p0, float t){ vec2 p = mod(p0 * 6.28318, 6.28318) - 250.; vec2 i = p; float c = 1.; for (int n = 0; n < 4; n++) { float tt = t * (1. - 3.5 / float(n + 1)); i = p + vec2(cos(tt - i.x) + sin(tt + i.y), sin(tt - i.y) + cos(tt + i.x)); c += 1. / length(vec2(p.x / (sin(i.x + tt) / .005), p.y / (cos(i.y + tt) / .005))); } c /= 4.; c = 1.17 - pow(c, 1.4); return clamp(pow(abs(c), 8.), 0., 1.); }
    void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 sand = vec3(.93, .86, .68) * (.85 + .15 * fbm(p * 30.)), deep = vec3(.03, .42, .5), shallow = vec3(.2, .78, .76);
    float d = smoothstep(.0, 1., q.y * .8 + .1 * fbm(p * 3.)); vec3 col = mix(mix(sand, shallow, .55), mix(shallow, deep, .6), d);
    float c = cau(p * 2.2 + vec2(0., uT * .01), uT * .35 + 23.); col += vec3(.9, 1., .95) * c * (.35 + .1 * uK) * (1. - d * .5);
    float rip = fbm(p * vec2(6., 3.) + vec2(uT * .1, -uT * .05)); col *= .9 + .2 * rip;
    float ray = pow(max(0., vn(vec2(p.x * 7. + p.y * 2. + uT * .1, 1.)) - .2), 2.) * smoothstep(.2, 1., q.y); col += vec3(.7, 1., .95) * ray * .25;
    col += vec3(1.) * stars(q + vec2(0., uT * .01), 40., uT * 3.) * .5; ${ml}`},dunes:{name:"Dunes",c:"#FF8A4C",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 col = mix(vec3(1., .5, .25), vec3(.85, .3, .38), smoothstep(.35, .62, q.y)); col = mix(col, vec3(.2, .12, .32), smoothstep(.6, 1., q.y));
    col += vec3(1.) * stars(q, 120., uT) * smoothstep(.75, .95, q.y) * .7;
    vec2 sp = (q - vec2(.5, .45)) * vec2(.474, 1.); float sd = length(sp); col += vec3(1., .75, .45) * (smoothstep(.07, .065, sd) * .9 + exp(-sd * 7.) * .5) * (.8 + .2 * uK);
    col = mix(col, col * vec3(.9, .7, .8), smoothstep(.5, .7, fbm(vec2(p.x * 2. + uT * .02, q.y * 12.))) * smoothstep(.45, .65, q.y) * .6);
    for (int i = 0; i < 4; i++) { float fi = float(i), base = .4 - fi * .085;
      float h = base + .05 * sin(p.x * (5. + fi * 2.) + fi * 2.) + .03 * sin(p.x * (11. + fi) - fi) + .02 * (fbm(vec2(p.x * 4. + fi * 9., 1.)) - .5);
      float sl = .05 * (5. + fi * 2.) * cos(p.x * (5. + fi * 2.) + fi * 2.) + .03 * (11. + fi) * cos(p.x * (11. + fi) - fi); float lit = smoothstep(-.35, .35, -sl) * .8 + .2 * smoothstep(.05, .0, h - q.y);
      vec3 dc = mix(vec3(.45, .16, .2), vec3(1., .58, .32), lit) * (1. - fi * .16) * (1. - .25 * smoothstep(0., .2, h - q.y));
      dc = mix(dc, col, .35 - fi * .1); float m = smoothstep(h + .002, h - .002, q.y); col = mix(col, dc + vec3(1., .7, .4) * exp(-(h - q.y) * 60.) * .25, m); }
    ${ml}`},neon:{name:"Neon rain",c:"#FF4FD8",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 col = mix(vec3(.02, .01, .05), vec3(.1, .03, .14), q.y);
    for (int k = 0; k < 3; k++) { float fk = float(k), sz = 9. - fk * 2.5; vec2 g = q * vec2(sz * .474, sz) + vec2(fk * 3.1, -uT * .01 * (fk + 1.)); vec2 id = floor(g); vec2 f = fract(g) - .5; float h = h21(id + fk * 11.);
      vec3 bc = h < .33 ? vec3(1., .2, .8) : h < .66 ? vec3(.2, .85, 1.) : vec3(1., .65, .2); float r = .18 + .2 * h21(id + 5.); vec2 o = (vec2(h21(id + 1.), h21(id + 2.)) - .5) * .5;
      float d = length(f - o); col += bc * (smoothstep(r, r * (.75 - fk * .2), d) * .7 + exp(-d * d / (r * r) * 2.) * .3) * step(.5, h21(id + 9.)) * (.32 - fk * .06) * (.8 + .2 * sin(uT * 2. + h * 30.)) * smoothstep(.18, .5, q.y); }
    float bid = floor(p.x * 14. + 7.), bh = .1 + .2 * h21(vec2(bid, 2.)) + .08 * step(.8, h21(vec2(bid, 5.))); float inb = step(q.y, bh);
    vec2 wg = vec2(p.x * 14. * 6., q.y * 90.); float win = step(.35, fract(wg.x)) * step(.4, fract(wg.y)) * step(.62, h21(floor(wg) + bid)) * (.6 + .4 * sin(uT * .5 + h21(floor(wg)) * 20.));
    vec3 wc = h21(floor(wg) + 3.) < .5 ? vec3(1., .75, .4) : vec3(.4, .9, 1.);
    col = mix(col, vec3(.01, .006, .02) + wc * win * .5 + vec3(.25, .05, .3) * smoothstep(bh - .05, bh, q.y) * .3, inb); col += vec3(.6, .15, .7) * exp(-max(q.y - bh, 0.) * 60.) * (1. - inb) * .12;
    vec2 rg = vec2(p.x * 160., q.y * 7. + uT * 2.4); float rid = floor(rg.x); float rf = fract(rg.y + h21(vec2(rid, 1.)) * 10.); col += vec3(.6, .7, .9) * smoothstep(.04, 0., abs(fract(rg.x) - .5) - .02) * smoothstep(.0, .05, rf) * smoothstep(.25, .05, rf) * step(.86, h21(vec2(rid, 3.))) * .35;
    ${Pi}`},forest:{name:"Forest light",c:"#C8F07A",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 col = mix(vec3(.01, .04, .02), vec3(.05, .14, .07), q.y); col = mix(col, vec3(.1, .2, .1), smoothstep(.0, .25, q.y) * smoothstep(.45, .2, q.y) * .5);
    vec2 lp = vec2(-.35, 1.15); vec2 dv = p - (lp - vec2(.5, .5)) * vec2(.474, 1.); float a = atan(dv.x, -dv.y);
    float rays = pow(vn(vec2(a * 22., uT * .15)), 3.) * smoothstep(1.4, .2, length(dv)) * (.55 + .45 * vn(vec2(a * 6. - uT * .08, 2.)));
    col += vec3(.85, 1., .6) * rays * (.5 + .12 * uK);
    float can = fbm(p * vec2(16., 12.) + vec2(uT * .01, 0.)) + (q.y - .8) * 1.4 + .12 * sin(p.x * 30.); float leaf = smoothstep(.46, .56, can); col = mix(col, vec3(.006, .02, .01) + vec3(.05, .12, .04) * fbm(p * 40.) * (1. - leaf * .7), leaf);
    float trunk = smoothstep(.02, .0, abs(p.x + .17 + .02 * sin(q.y * 6.)) - .025) + smoothstep(.02, .0, abs(p.x - .2 - .015 * sin(q.y * 5. + 1.)) - .035); col = mix(col, vec3(.01, .02, .012), clamp(trunk, 0., 1.) * .85);
    vec2 mq = q + vec2(sin(uT * .3 + q.y * 9.) * .01, -uT * .008); col += vec3(1., .95, .6) * stars(mq, 60., uT * 2.) * .9 * (.3 + rays * 3.);
    col += vec3(.2, .35, .22) * smoothstep(.3, .0, q.y) * (.5 + .5 * fbm(p * 4. + vec2(uT * .05, 0.))) * .5; ${Pi}`},forge:{name:"Forge",c:"#FF6A1F",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    float hz = .42, z = 1. / max(hz - q.y, .012); vec2 fp = vec2(p.x * z * .9, z * .35 + uT * .0);
    float n = fbm(p * 7.); vec3 col = mix(vec3(.03, .018, .015), vec3(.1, .06, .05), fbm(p * vec2(4., 2.) + vec2(0., uT * .03)));
    col += vec3(.8, .25, .05) * exp(-abs(q.y - hz) * 9.) * .35 * (.85 + .2 * uK);
    if (q.y < hz) { vec3 v = vor(fp); float edge = v.y - v.x, fog = smoothstep(.0, .12, hz - q.y), pulse = .7 + .3 * sin(uT * 1.6 + v.z * 20.);
      vec3 rock = mix(vec3(.04, .025, .02), vec3(.13, .08, .06), fbm(fp * 2.)) * (.6 + .4 * smoothstep(.0, .3, edge));
      vec3 lava = mix(vec3(1., .25, .02), vec3(1., .78, .3), smoothstep(.05, .0, edge)) * smoothstep(.09, .0, edge) * pulse * (.85 + .2 * uK);
      col = mix(col, rock + lava * fog, fog) + vec3(1., .35, .06) * (1. - fog) * .25; }
    for (int k = 0; k < 2; k++) { float fk = float(k); vec2 eq = q + vec2(.02 * sin(uT * .9 + q.y * 12. + fk), -uT * (.05 + fk * .03)); col += vec3(1., .55, .15) * stars(eq + fk * 2.7, 55. + fk * 30., uT * 4.) * 1.3 * smoothstep(1., .3, q.y); }
    float sm = fbm(p * vec2(3., 2.) + vec2(uT * .04, -uT * .08)); col += vec3(.3, .14, .08) * smoothstep(.45, .8, sm) * smoothstep(.4, .9, q.y) * .35; ${Pi}`},crystal:{name:"Geode",c:"#B58CFF",frag:`${In} void main(){ vec2 q = scr(), p = (q - vec2(.5, .51)) * vec2(.474, 1.);
    float r = length(p), a = atan(p.y, p.x); vec3 col = vec3(.03, .01, .05) + vec3(.35, .18, .55) * exp(-r * r * 10.) * (.7 + .25 * uK);
    for (int k = 0; k < 3; k++) { float fk = float(k), N = 15. + fk * 8., sa = 6.28318 / N; float aa = a + fk * .37 + .02 * sin(uT * .2 + fk); float sec = floor(aa / sa), f = fract(aa / sa) - .5;
      float h = h21(vec2(sec, fk)), R0 = .62 - fk * .08, L = .16 + .18 * h, tip = R0 - L; float w = .55 * (r - tip) / L; float inside = step(tip, r) * step(abs(f), w);
      vec3 cc = mix(vec3(.22, .08, .42), vec3(.62, .4, .95), .35 + .5 * h) * (f < 0. ? 1.15 : .55) * (.55 + .45 * smoothstep(R0, tip, r)) * (1. - fk * .18);
      cc += vec3(.9, .8, 1.) * smoothstep(.04, .0, abs(f - w * (f < 0. ? -1. : 1.) * .98) * r) * .25 + vec3(.95, .9, 1.) * smoothstep(.03, .0, abs(f) * r) * smoothstep(tip + .08, tip, r) * .3;
      cc += vec3(1.) * pow(max(0., sin(h * 60. + uT * .6)), 40.) * smoothstep(tip + .05, tip, r) * .6;
      col = mix(col, cc, inside); }
    col += vec3(.95, .9, 1.) * stars(q, 110., uT * 3.) * smoothstep(.1, .3, r) * .6; col *= 1. - smoothstep(.45, .95, r) * .6; ${Pi}`},sakura:{name:"Blossom dusk",c:"#FF8FB8",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 col = mix(vec3(1., .72, .7), vec3(.86, .58, .78), smoothstep(.25, .6, q.y)); col = mix(col, vec3(.42, .36, .66), smoothstep(.6, 1., q.y));
    float cl = fbm(vec2(p.x * 3. + uT * .015, q.y * 6.)); col = mix(col, vec3(1., .85, .88), smoothstep(.55, .78, cl) * .35);
    for (int k = 0; k < 2; k++) { float fk = float(k), sz = 7. + fk * 4.; vec2 g = q * vec2(sz * .474, sz) + vec2(fk * 2.3 + uT * .006, uT * .004); vec2 id = floor(g), f = fract(g) - .5; float h = h21(id + fk * 7.);
      float d = length(f - (vec2(h21(id + 1.), h21(id + 2.)) - .5) * .5); float r = .2 + .15 * h; col = mix(col, mix(vec3(1., .93, .96), vec3(1., .7, .85), h), smoothstep(r, r * .4, d) * step(.4, h) * (.45 - fk * .15) * smoothstep(.45, .9, q.y)); }
    float br = abs(q.y - (.93 - .25 * pow(max(q.x - .0, 0.), 1.5) + .02 * sin(q.x * 18.))) - .006 * (1.3 - q.x); float twig = abs(q.y - (.86 - .4 * (q.x - .25) + .015 * sin(q.x * 30.))) - .003;
    col = mix(col, vec3(.2, .1, .14), smoothstep(.004, .0, br) * step(q.x, .9)); col = mix(col, vec3(.22, .11, .15), smoothstep(.003, .0, twig) * step(.25, q.x) * step(q.x, .55));
    for (int k = 0; k < 2; k++) { float fk = float(k); vec2 pq = q + vec2(uT * .02 + sin(uT * .7 + q.y * 7. + fk) * .015, uT * (.035 + fk * .02)); vec2 g = pq * vec2(14. * .474, 14.) + fk * 5.; vec2 id = floor(g), f = fract(g) - .5; float h = h21(id);
      float ang = uT * (1. + h * 2.) + h * 6.; mat2 m = mat2(cos(ang), -sin(ang), sin(ang), cos(ang)); vec2 pf = m * (f - (vec2(h, h21(id + 4.)) - .5) * .5); col = mix(col, vec3(1., .8, .88), smoothstep(.06, .03, length(pf * vec2(1., 2.))) * step(.72, h) * .9); }
    col += vec3(1., .9, .7) * exp(-length((q - vec2(.5, .3)) * vec2(.474, 1.)) * 5.) * .25; ${ml}`},storm:{name:"Thunderhead",c:"#9FC3FF",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    float t = uT * .05; vec2 w = vec2(fbm(p * 2.5 + vec2(t, 0.)), fbm(p * 2.5 + vec2(3.1, -t)));
    float c1 = fbm(p * vec2(3., 4.) + w * 1.6 + vec2(t * 2., 0.)), c2 = fbm(p * vec2(6., 8.) - w + vec2(-t * 3., t));
    vec3 col = mix(vec3(.05, .06, .09), vec3(.4, .44, .55), smoothstep(.25, .85, c1)); col = mix(col, vec3(.55, .58, .68), smoothstep(.5, .8, c2) * .5 * c1); col += vec3(.25, .3, .4) * smoothstep(.62, .75, c1) * (1. - c2) * .5;
    float fl = pow(max(0., sin(uT * 1.1) * sin(uT * 2.7 + 1.) * sin(uT * .6 + 2.)), 4.) * 9. + max(0., uK - .3) * .5 + .15;
    vec2 lp = vec2(.12 * sin(floor(uT * .4) * 3.7), .2); float ld = length(p - lp);
    col += vec3(.6, .7, 1.) * c1 * c1 * exp(-ld * 3.) * fl * .5 + vec3(.5, .6, .9) * c2 * exp(-ld * 6.) * fl * .25;
    vec2 rg = vec2(p.x * 90. + q.y * 8., q.y * 4. + uT * 3.); float rid = floor(rg.x), rf = fract(rg.y + h21(vec2(rid, 1.)) * 9.); col += vec3(.5, .6, .75) * step(.82, h21(vec2(rid, 3.))) * smoothstep(.2, .0, rf) * smoothstep(.06, 0., abs(fract(rg.x) - .5) - .03) * .12;
    float bx = p.x - lp.x - .03 * (fbm(vec2(q.y * 18., floor(uT * .4))) - .5) * 3.; float bolt = smoothstep(.004, .0, abs(bx)) * step(q.y, .95) * step(lp.y + .5, q.y) * step(.6, fl); col += vec3(.85, .9, 1.) * bolt * fl * .3 + vec3(.4, .5, .9) * exp(-abs(bx) * 60.) * step(lp.y + .5, q.y) * step(.6, fl) * .2;
    col *= 1. - smoothstep(.3, .9, length(p * vec2(1.6, .9))) * .45; ${Pi}`},velvet:{name:"Velvet stage",c:"#FF4D5E",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    vec3 col = mix(vec3(.02, .008, .01), vec3(.07, .02, .03), q.y);
    vec2 sd = p - vec2(0., .58); float cone = smoothstep(.02, -.02, abs(sd.x) - (.03 + (.58 - p.y) * .2)) * smoothstep(-.6, .5, p.y);
    col += vec3(1., .9, .75) * cone * (.12 + .06 * vn(vec2(p.x * 20., uT * .3))) * (.9 + .2 * uK);
    float pool = exp(-pow(length((p - vec2(0., -.32)) * vec2(1., 3.2)) * 5., 2.)); col += vec3(1., .8, .6) * pool * .35;
    col += vec3(.9, .85, .8) * stars(q + vec2(sin(uT * .2) * .02, uT * .004), 70., uT) * cone * 3.;
    float open = .17 + .02 * sin(uT * .3); float side = abs(p.x); float sway = .006 * sin(uT * .8 + q.y * 3.);
    float pl = sin((side + sway) * 95. + fbm(vec2(side * 6., q.y * 2.)) * 2.); float fold = .5 + .5 * pl; float cur = smoothstep(open - .01, open + .01, side + (1. - q.y) * .05 * (1. - q.y));
    vec3 vc = vec3(.5, .04, .08) * (.25 + .9 * pow(fold, 2.)) + vec3(1., .5, .5) * pow(fold, 18.) * .25; vc *= .65 + .35 * smoothstep(0., .9, q.y);
    col = mix(col, vc, cur);
    float val = .9 - .025 * cos(p.x * 40.) ; float vm = smoothstep(val - .005, val + .005, q.y); vec3 vcol = vec3(.45, .03, .07) * (.5 + .5 * fbm(vec2(p.x * 30., q.y * 3.))) + vec3(1., .75, .3) * smoothstep(.006, .0, abs(q.y - val - .006)) * .6;
    col = mix(col, vcol, vm); col += vec3(1., .6, .5) * exp(-abs(side - open) * 120.) * .15 * (1. - cur); ${Pi}`},nebula:{name:"Nebula",c:"#FF7BD5",frag:`${In} void main(){ vec2 q = scr(), p = (q - .5) * vec2(.474, 1.);
    float t = uT * .01; vec2 w = vec2(fbm(p * 3. + t), fbm(p * 3. + 5.2 - t)); float n1 = fbm(p * 2.2 + w * 1.8 + t), n2 = fbm(p * 4. - w * 1.2 + 9.);
    vec3 col = vec3(.008, .006, .02) + vec3(.02, .015, .05) * q.y;
    float band = exp(-pow((p.y - p.x * 1.1 - .02) * 3.2, 2.));
    col += mix(vec3(.75, .15, .65), vec3(1., .5, .22), smoothstep(.3, .8, n2)) * pow(n1, 2.2) * 2.4 * (.35 + band);
    col += vec3(.12, .55, .8) * pow(n2, 2.6) * 1. * (1. - band * .5);
    col *= 1. - smoothstep(.55, .8, fbm(p * 7. + w * 2.)) * band * .7;
    col += vec3(1., .95, .9) * band * .1 * n1;
    col += vec3(.85, .9, 1.) * (stars(q, 220., uT) * .8 + stars(q + 1.7, 110., uT * .7) + stars(q + 4.1, 45., uT * .5) * 1.4) * (.6 + band);
    ${Pi}`}},t2=Object.keys(zy);var Hy={b:null};var Lt=Math.PI*2,nn={expo:n=>n>=1?1:1-Math.pow(2,-10*n),quart:n=>1-Math.pow(1-n,4),back:n=>1+(1.9+1)*Math.pow(n-1,3)+1.9*Math.pow(n-1,2),sine:n=>.5-.5*Math.cos(Math.PI*n)};function Pl(n,t){n.free=t;let e=n.s;if(t){n.jit=0;for(let i=Li.length-1;i>=0;i--){let s=Li[i];(s.o===e||s.o===n&&("x"in s.to||"y"in s.to||"z"in s.to))&&(Li.splice(i,1),s.res())}}t||(e.vrx=e.vry=e.vfl=e.vrz=0,n.flipT=e.fl)}function Ce(n,t,e=ee.lin){let i={p:0},s=vp(()=>t(e(i.p)));return De(i,{p:1},n,ee.lin).then(()=>{s(),t(e(1))})}var Ci=(n,t=0)=>Math.ceil((n+.02)/Lt)*Lt+t*Lt;function ri(n,t,e){let i=Math.cos(n.s.fl)*Math.cos(n.s.rx);i<-.2&&(t._back=!0),t._back&&!t._faced&&i>.3&&(e==null||Math.abs(e-n.s.fl)<1.3)&&(t._faced=!0,t.onFace?.())}function Ra(n,t,e){return El(()=>{let i=Gt(0,Lt),s=Gt(.5,1.3);vs({kind:"arc",bk:1,x:n.x,y:n.y,rx:118*n.s.sc*Gt(1,1.25),ry:160*n.s.sc*Gt(1,1.2),a0:i,a1:i+s,color:Math.random()<.3?"#FFFFFF":e,w:Gt(1.2,3),a:Gt(.35,.7),life:Gt(.18,.32)})},40,t/1e3)}var wt=(n,t,e)=>n+(t-n)*e;function ai(n,t){let e=n.s;return{x0:n.x,y0:n.y,z0:n.z,sc0:e.sc,fl0:e.fl,rx0:e.rx,ry0:e.ry,rz0:e.rz,X:t.x??n.x,Y:t.y??n.y,Z:t.z??0,S1:t.sc??1,col:n.C}}var Gf={whip:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0,1);Z.play("suck",.25),await Ce(240,s=>{n.s.fl=e.fl0-.55*s,n.s.sc=e.sc0*(1-.06*s),n.s.rz=e.rz0+.1*s},ee.out),Z.play("whoosh"),Ra(n,520,e.col),await Ce(1e3,s=>{n.s.fl=wt(e.fl0-.55,i,s),n.s.sc=wt(e.sc0*.94,e.S1,s),n.s.rz=(e.rz0+.1)*(1-s),n.x=wt(e.x0,e.X,s),n.y=wt(e.y0,e.Y,s),n.z=wt(e.z0,e.Z,s),ri(n,t,i)},nn.expo)},toss:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0),s=e.rx0+2*Lt;Z.play("swoop"),Ra(n,700,e.col),await Ce(1050,r=>{n.y=wt(e.y0,e.Y,r)-600*r*(1-r),n.x=wt(e.x0,e.X,r),n.z=wt(e.z0,e.Z,r)+1.6*Math.sin(Math.PI*r),n.s.rx=wt(e.rx0,s,nn.quart(r)),n.s.fl=wt(e.fl0,i,nn.sine(r)),n.s.sc=wt(e.sc0,e.S1,r),ri(n,t,i)}),n.s.rx=0,Z.play("thud"),Ys(.2),n.s.sc=e.S1*.92,De(n.s,{sc:e.S1},320,nn.back)},vortex:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0,1);Z.play("riser",.5),Ra(n,900,e.col),await Ce(1250,s=>{n.s.rz=e.rz0+2*Lt*nn.quart(s),n.s.fl=wt(e.fl0,i,nn.quart(s)),n.s.sc=wt(e.sc0*.45,e.S1,nn.back(s)),n.z=wt(e.z0-2,e.Z,ee.out(s)),n.x=wt(e.x0,e.X,s),n.y=wt(e.y0,e.Y,s),ri(n,t,i)}),n.s.rz=e.rz0},page:async(n,t)=>{let e=ai(n,t),i=Ii/2*e.sc0,s=e.x0-i*100;Z.play("paper"),await Ce(820,r=>{let a=-Math.PI*r;n.s.fl=e.fl0+a,n.x=s+i*100*Math.cos(a),n.z=e.z0-i*Math.sin(a)*1.1,n.s.rz=.05*Math.sin(Math.PI*r),ri(n,t)},nn.sine),Z.play("whoosh",.4),await Ce(520,r=>{n.x=wt(s-i*100,e.X,r),n.y=wt(e.y0,e.Y,r),n.z=wt(e.z0,e.Z,r),n.s.sc=wt(e.sc0,e.S1,r),n.s.rz=-.06*Math.sin(Math.PI*r)},nn.back)},slam:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0);Z.play("suck",.3),await Ce(300,s=>{n.z=wt(e.z0,e.z0+3.4,s),n.s.rx=wt(e.rx0,-.25,s),n.x=wt(e.x0,e.X,s),n.y=wt(e.y0,e.Y,s)},ee.in),Z.play("whoosh"),await Ce(200,s=>{n.s.fl=wt(e.fl0,i,s),ri(n,t,i)},ee.out),_l(90),await Ce(650,s=>{n.z=wt(e.z0+3.4,e.Z,s),n.s.rx=wt(-.25,0,s),n.s.sc=wt(e.sc0,e.S1,s)},nn.back)},drop:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0);Z.play("swoop"),await Ce(320,s=>{n.y=wt(e.y0,-180,s),n.s.rx=wt(e.rx0,.5,s)},ee.in),n.x=e.X,n.s.sc=e.S1,n.z=e.Z,await Ce(420,s=>{n.y=wt(-180,e.Y,s),n.s.rx=wt(.5,-.15,s)},ee.in2),Z.play("thud"),Ys(.3),_l(50),await Ce(380,s=>{n.y=e.Y-46*Math.sin(Math.PI*s),n.s.fl=wt(e.fl0,i,ee.out(s)),n.s.rx=wt(-.15,.1,s),ri(n,t,i)}),await Ce(260,s=>{n.s.rx=wt(.1,0,s)},nn.back)},tease:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0,1);for(let[s,r]of[[.6,0],[-.75,1]])Z.play("tick",900+r*300),await Ce(170,a=>{n.s.fl=e.fl0+s*a,n.s.sc=e.sc0*(1+.03*a)},ee.out),await Ce(170,a=>{n.s.fl=e.fl0+s*(1-a),n.s.sc=e.sc0*(1.03-.03*a)},ee.inOut);Z.play("whoosh"),Ra(n,500,e.col),await Ce(1100,s=>{n.s.fl=wt(e.fl0,i,s),n.x=wt(e.x0,e.X,s),n.y=wt(e.y0,e.Y,s),n.z=wt(e.z0,e.Z,s),n.s.sc=wt(e.sc0,e.S1,s),ri(n,t,i)},nn.quart)},float:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0);Z.play("shimmer"),await Ce(1600,s=>{n.s.fl=wt(e.fl0,i,nn.sine(s)),n.y=wt(e.y0,e.Y,s)-30*Math.sin(Math.PI*s),n.x=wt(e.x0,e.X,s),n.z=wt(e.z0,e.Z,s)+.6*Math.sin(Math.PI*s),n.s.rx=.28*Math.sin(Math.PI*s),n.s.rz=-.12*Math.sin(Lt*s),n.s.sc=wt(e.sc0,e.S1,s),ri(n,t,i)})},spiral:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0,1);Z.play("whoosh"),Ra(n,900,e.col),await Ce(1300,s=>{let r=s*3*Math.PI,a=1-nn.quart(s);n.x=e.X+80*Math.sin(r)*a+(e.x0-e.X)*(1-s),n.y=e.Y-46*(1-Math.cos(r))*a+(e.y0-e.Y)*(1-s),n.s.fl=wt(e.fl0,i,nn.quart(s)),n.s.rz=.45*Math.sin(r)*a,n.z=wt(e.z0,e.Z,s),n.s.sc=wt(e.sc0,e.S1,s),ri(n,t,i)}),n.s.rz=0},snap:async(n,t)=>{let e=ai(n,t),i=Ci(e.fl0);Z.play("suck",.2),await Ce(150,s=>{n.s.sc=e.sc0*(1-.12*s),n.x=wt(e.x0,e.X,s),n.y=wt(e.y0,e.Y,s)},ee.out),Z.play("whoosh"),await Ce(230,s=>{n.s.sc=wt(e.sc0*.88,e.S1*1.2,s),n.s.fl=wt(e.fl0,i,s),n.z=wt(e.z0,e.Z+.6,s),ri(n,t,i)},ee.out),_l(70),await Ce(420,s=>{n.s.sc=wt(e.S1*1.2,e.S1,s),n.z=wt(e.Z+.6,e.Z,s)},nn.back)}};async function Vy(n,t={}){let e=t.spin||Ul[ep]?.spin||"whip",i=Gf[e]||Gf.whip;Pl(n,!0);let s=()=>{let a=t.x??n.x,o=t.y??n.y;if(Ty(a,o,we,{white:!1}),n.glowCol.set(n.C),n.glowA=1.2,De(n,{glowA:.42},1300,ee.out),t.rays!==!1&&we>=3){let l=sy(a,o,n.C,700);It(l,[{opacity:0,transform:"scale(.4)"},{opacity:1,transform:"scale(1)"}],700)}Zs(()=>fp(n),260),t.onFace?.()},r={...t,onFace:s};await i(n,r),r._faced||s(),Pl(n,!1)}var Gy="classic",yp=()=>Ul[ep]?.look||Gy;function Pa(n,t,e,i){let s=re[t],r=i.x??13,a=i.align||"left",o=i.ink||"#F4F2EF",l=i.sub||"#CFE0DE",c=i.chipBg||"rgba(244,242,239,.09)";n.textAlign=a,n.fillStyle=o,n.font=`${i.weight||500} ${i.size||16}px ${Re}`,n.textBaseline="alphabetic",i.shadow&&(n.shadowColor=i.shadow,n.shadowBlur=6,n.shadowOffsetY=1);let h=a==="center"?100:r;{let p=Gv(n,a==="center"?172:176);n.fillText(p[0],h,i.y),p[1]&&n.fillText(p[1],h,i.y+(i.size||16)*1.2)}n.textAlign="left",n.shadowColor="transparent",n.shadowBlur=0,n.shadowOffsetY=0;let u=(p,T,M)=>{p.strokeStyle=l,p.lineWidth=1.2,Nt(p,T-5,M-3.5,10,7.5,1.6),p.stroke(),p.beginPath(),p.arc(T,M,2,0,7),p.stroke()},d=[[Di.kind,{icon:Wv(l),fg:l,bg:c,stroke:i.chipStroke}],[Di.no,{fg:l,bg:c,stroke:i.chipStroke}]];qv(d);let f=i.y+(i.size||16)*1.2+10,g=11;for(let p of d)Object.assign(p[1],{size:g,h:19,pad:6});let x=r;if(a==="center"){let p=Pe(10,10).getContext("2d"),T=0;for(let[M,y]of d)p.font=`500 ${g}px ${Re}`,T+=p.measureText(M).width+12+(y.icon?g+3:0)+4;x=100-(T-4)/2}for(let[p,T]of d)x+=gs(n,x,f,p,T)+4;x=a==="center"?14:r-1;let m=f+24;for(let p=0;p<e.length;p++){let T=e[p],M=T.k==="space",y=(M?"":"#")+T.label;n.font=`${M?600:500} 10.5px ${Re}`;let D=n.measureText(y).width+12+(M?13.5:0);if(x+D>188){gs(n,x,m,"+"+(e.length-p),{bg:i.tagBg||"rgba(244,242,239,.14)",fg:i.tagFg||"#E8ECEB",size:10.5,pad:6});break}x+=gs(n,x,m,y,M?{bg:i.spaceBg||"#F4F2EF",fg:i.spaceFg||"#0E333A",weight:600,size:10.5,pad:6,icon:(S,C,P)=>{S.fillStyle=T.col,S.beginPath(),S.arc(C,P,3,0,7),S.fill()}}:{bg:i.tagBg||"rgba(244,242,239,.14)",fg:i.tagFg||"#E8ECEB",size:10.5,pad:6})+4}}function yu(n,t,e,i={}){let s=re[t];n.font=`700 12px ${Re}`,n.textBaseline="middle",n.textAlign="left",n.fillStyle=i.label||s,n.fillText(ki[t],i.x??16,21);for(let r=0;r<Se[t];r++)Js(n,(i.sx??184)-(Se[t]-1-r)*13,21,5.6,r<e?i.on||s:i.off||"rgba(244,242,239,.14)")}var oi=()=>{let t=Pe(800,1136),e=t.getContext("2d");return e.scale(4,4),[t,e]},Ia=()=>{let n=Pe(400,568),t=Pe(400,568),e=n.getContext("2d"),i=t.getContext("2d");return e.scale(2,2),i.scale(2,2),[n,t,e,i]};function Wy(n,t,e=7,i=11){Nt(n,0,0,200,284,16);let s=n.createLinearGradient(0,0,200,284);[t[0],t[1],t[2],t[3],t[1],t[4],t[0]].forEach((r,a)=>s.addColorStop(a/6,r)),n.fillStyle=s,n.fill()}function qy(n,t,e,i,s){let r=i/2;n.beginPath(),n.moveTo(t,e+s),n.lineTo(t,e+r),n.arc(t+r,e+r,r,Math.PI,0),n.lineTo(t+i,e+s),n.closePath()}function Du(n,t){let e=new Xe,i=n/2;return e.moveTo(-n/2,-t/2),e.lineTo(n/2,-t/2),e.lineTo(n/2,t/2-i),e.absarc(0,t/2-i,i,0,Math.PI,!1),e.lineTo(-n/2,-t/2),e}var _u={};function Xy(n="color"){if(n==="color"||!un.complete)return _s;if(_u[n])return _u[n];let t=300,e=400,i=Pe(t,e),s=i.getContext("2d");s.drawImage(un,0,0,t,e);let r=s.getImageData(0,0,t,e),a=r.data,[o,l]=n==="duo"?[[6,26,30],[214,236,230]]:[[40,26,14],[246,232,204]];for(let h=0;h<a.length;h+=4){let u=(a[h]*.3+a[h+1]*.59+a[h+2]*.11)/255;u=u*u*(3-2*u);for(let d=0;d<3;d++)a[h+d]=o[d]+(l[d]-o[d])*u}s.putImageData(r,0,0);let c=Tt(i);return c.keep=!0,_u[n]=c}function $y(n){let t=n.shape==="circle"?new ya(n.w/2,64):n.shape==="arch"?new Dr(Du(n.w,n.h),24):La(n.w,n.h,n.r??.1),e=t.attributes.position,i=t.attributes.uv,s=600/800,r=n.w/n.h,a=r>s?1:r/s,o=r>s?s/r:1;for(let l=0;l<e.count;l++)i.setXY(l,.5+e.getX(l)/n.w*a,.5+e.getY(l)/n.h*o);return t}function Yy(n,t){let e=n.rim??.035,i;if(n.shape==="circle"){i=new Xe,i.absarc(0,0,n.w/2+e,0,Lt,!1);let s=new _n;s.absarc(0,0,n.w/2,0,Lt,!0),i.holes.push(s)}else if(n.shape==="arch"){i=Du(n.w+e*2,n.h+e*2);let s=Du(n.w,n.h),r=new _n(s.getPoints(48).reverse());i.holes.push(r)}else i=new Xe,Xr(i,n.w+e*2,n.h+e*2,(n.r??.1)+e*.8),i.holes.push(Xr(new _n,n.w,n.h,n.r??.1));return new Ot(new ns(i,{depth:n.depth??.045,bevelEnabled:!1,curveSegments:24}),t)}function Zy(n,t){let e=100+-t.w/2*100,i=142-(t.y+t.h/2)*100,s=t.w*100,r=t.h*100;return t.shape==="circle"?(n.beginPath(),n.arc(100,142-t.y*100,s/2,0,7)):t.shape==="arch"?qy(n,e,i,s,r):Nt(n,e,i,s,r,(t.r??.1)*100),[e,i,s,r]}var Il={classic:{art:{w:1.74,h:1.42,y:.37,r:.1,bezel:!0},face:(n,t=[],e=0)=>vy(n,t,e),maps:n=>yy(n),back:n=>Vf(n)},full:{art:{w:1.88,h:2.72,y:0,r:.12,bezel:!1,z:.02},sparkle:1.2,face:n=>{let[t,e]=oi();return Wy(e,bs[n]),Nt(e,6,6,188,272,12),e.fillStyle="#05090B",e.fill(),t},over:(n,t=[],e=0)=>{let[i,s]=oi();s.save(),Nt(s,6,6,188,272,12),s.clip();let r=s.createLinearGradient(0,128,0,284);r.addColorStop(0,"rgba(3,10,12,0)"),r.addColorStop(.3,"rgba(3,10,12,.78)"),r.addColorStop(.55,"rgba(3,10,12,.9)"),r.addColorStop(1,"rgba(3,10,12,.96)"),s.fillStyle=r,s.fillRect(0,128,200,156);let a=s.createLinearGradient(0,6,0,46);return a.addColorStop(0,"rgba(3,10,12,.7)"),a.addColorStop(1,"rgba(3,10,12,0)"),s.fillStyle=a,s.fillRect(0,6,200,40),s.restore(),yu(s,n,e),Pa(s,n,t,{y:204,weight:600,shadow:"rgba(0,0,0,.85)"}),s.save(),Nt(s,6,6,188,272,12),s.lineWidth=1,s.strokeStyle="rgba(255,255,255,.22)",s.stroke(),s.restore(),i},maps:n=>{let[t,e,i,s]=Ia();if(i.fillStyle="rgb(0,55,255)",i.fillRect(0,0,200,284),Nt(i,6,6,188,272,12),i.fillStyle="rgb(0,170,0)",i.fill(),s.fillStyle="#000",s.fillRect(0,0,200,284),n>=3){s.save(),Nt(s,6,6,188,272,12),s.clip();for(let r=-40;r<70;r++)s.strokeStyle=`rgba(255,255,255,${.25+.25*Math.sin(r*.7)})`,s.lineWidth=2.2,s.beginPath(),s.moveTo(r*6,0),s.lineTo(r*6-200,284),s.stroke();s.restore()}return{orm:Tt(t,!1),foil:Tt(e,!1)}},back:n=>Vf(n)},noir:{art:{w:1.5,h:1.36,y:.36,shape:"arch",bezel:!0,rim:.03},edge:"#1B1A18",edgeRough:.35,bezelCol:"#E3B866",face:(n,t=[],e=0)=>{let[i,s]=oi(),r=re[n],a="#E9C277";Nt(s,0,0,200,284,16);let o=s.createLinearGradient(0,0,200,284);o.addColorStop(0,"#16171A"),o.addColorStop(.5,"#0B0C0E"),o.addColorStop(1,"#141416"),s.fillStyle=o,s.fill(),s.strokeStyle=a,s.lineWidth=1.2,Nt(s,7,7,186,270,10),s.stroke(),s.lineWidth=.5,Nt(s,11,11,178,262,8),s.stroke(),s.save(),s.beginPath(),s.rect(0,34,200,250),s.clip(),s.translate(100,113);for(let l=0;l<=22;l++){let c=Math.PI+.25+l/22*(Math.PI-.5);s.globalAlpha=l%2?.35:.7,s.beginPath(),s.moveTo(Math.cos(c)*82,Math.sin(c)*82),s.lineTo(Math.cos(c)*(l%2?88:93),Math.sin(c)*(l%2?88:93)),s.stroke()}return s.restore(),s.globalAlpha=1,yu(s,n,e,{label:"#F6D99A",on:r,off:"rgba(233,194,119,.2)",x:34,sx:166}),s.strokeStyle=a,s.lineWidth=.6,s.beginPath(),s.moveTo(60,184),s.lineTo(140,184),s.stroke(),ci(s,100,184,8,Fi.gold),Pa(s,n,t,{y:202,align:"center",ink:"#FFEBC6",sub:"#E2CFA5",size:15,weight:600,chipBg:"rgba(233,194,119,.08)",chipStroke:"rgba(233,194,119,.35)",tagBg:"rgba(233,194,119,.12)",tagFg:"#F3DDB0",spaceBg:"#E9C277",spaceFg:"#1A1408"}),i},maps:n=>{let[t,e,i,s]=Ia();i.fillStyle="rgb(0,120,0)",i.fillRect(0,0,200,284);let r=(a,o)=>{a.strokeStyle=o,a.lineWidth=1.6,Nt(a,7,7,186,270,10),a.stroke(),a.lineWidth=1,Nt(a,11,11,178,262,8),a.stroke(),a.save(),a.beginPath(),a.rect(0,34,200,250),a.clip(),a.translate(100,113),a.lineWidth=1.4;for(let l=0;l<=22;l++){let c=Math.PI+.25+l/22*(Math.PI-.5);a.beginPath(),a.moveTo(Math.cos(c)*82,Math.sin(c)*82),a.lineTo(Math.cos(c)*(l%2?88:93),Math.sin(c)*(l%2?88:93)),a.stroke()}a.restore()};return r(i,"rgb(0,40,255)"),s.fillStyle="#000",s.fillRect(0,0,200,284),n>=3&&r(s,"#FFF"),{orm:Tt(t,!1),foil:Tt(e,!1)}},backOrm:()=>gl(.4,.25),back:n=>{let[t,e]=oi(),i="#E9C277";Nt(e,0,0,200,284,16),e.fillStyle="#0D0E10",e.fill(),e.strokeStyle=i,e.lineWidth=1.2,Nt(e,7,7,186,270,10),e.stroke(),e.save(),e.translate(100,142);for(let s=0;s<48;s++){let r=s/48*Lt;e.globalAlpha=s%2?.3:.65,e.lineWidth=.6,e.beginPath(),e.moveTo(Math.cos(r)*34,Math.sin(r)*34),e.lineTo(Math.cos(r)*(s%2?70:92),Math.sin(r)*(s%2?70:92)),e.stroke()}return e.globalAlpha=1,e.lineWidth=1,e.beginPath(),e.arc(0,0,30,0,7),e.stroke(),ci(e,0,0,34,Fi.gold),e.restore(),e.fillStyle=i,e.font=`500 13px ${Re}`,e.textAlign="center",e.fillText("Inventory",100,258),t}},celestial:{art:{w:1.26,h:1.26,y:.44,shape:"circle",bezel:!0,rim:.04},edge:"#C9A55A",bezelCol:"#D9B76A",face:(n,t=[],e=0)=>{let[i,s]=oi(),r=re[n];Nt(s,0,0,200,284,16);let a=s.createRadialGradient(100,92,10,100,120,220);a.addColorStop(0,"#16305A"),a.addColorStop(.55,"#0B1A36"),a.addColorStop(1,"#050C1C"),s.fillStyle=a,s.fill();let o=ui(7);for(let c=0;c<140;c++)s.globalAlpha=.2+o()*.7,s.fillStyle="#E8F0FF",s.beginPath(),s.arc(o()*200,o()*284,o()*.9+.2,0,7),s.fill();s.globalAlpha=1,s.save(),s.beginPath(),s.rect(0,36,200,248),s.clip(),s.strokeStyle="rgba(233,212,160,.35)",s.lineWidth=.5;for(let c of[72,78])s.beginPath(),s.arc(100,98,c,0,7),s.stroke();s.translate(100,98);for(let c=0;c<72;c++){let h=c/72*Lt,u=c%6?3:6;s.beginPath(),s.moveTo(Math.cos(h)*78,Math.sin(h)*78),s.lineTo(Math.cos(h)*(78+u),Math.sin(h)*(78+u)),s.stroke()}s.restore();let l=[[150,160],[166,150],[176,164],[188,150]];return s.strokeStyle="rgba(233,212,160,.55)",s.lineWidth=.6,s.beginPath(),l.forEach(([c,h],u)=>u?s.lineTo(c,h):s.moveTo(c,h)),s.stroke(),l.forEach(([c,h])=>Js(s,c,h,2.2,"#F3E2B0")),yu(s,n,e,{label:"#F3E2B0",on:r,off:"rgba(233,212,160,.22)"}),Pa(s,n,t,{y:194,align:"center",ink:"#F4F2EF",sub:"#C9D6EA",size:15,chipBg:"rgba(201,214,234,.08)",chipStroke:"rgba(233,212,160,.3)",tagBg:"rgba(201,214,234,.12)",tagFg:"#E8F0FF"}),i},maps:n=>{let[t,e,i,s]=Ia();i.fillStyle="rgb(0,110,0)",i.fillRect(0,0,200,284),s.fillStyle="#000",s.fillRect(0,0,200,284);let r=(a,o,l)=>{a.strokeStyle=o,a.lineWidth=l,a.save(),a.beginPath(),a.rect(0,36,200,248),a.clip();for(let h of[72,78])a.beginPath(),a.arc(100,98,h,0,7),a.stroke();a.restore();let c=[[150,160],[166,150],[176,164],[188,150]];a.beginPath(),c.forEach(([h,u],d)=>d?a.lineTo(h,u):a.moveTo(h,u)),a.stroke(),a.fillStyle=o,c.forEach(([h,u])=>{a.beginPath(),a.arc(h,u,2.6,0,7),a.fill()})};if(r(i,"rgb(0,50,255)",1.4),n>=3){r(s,"#FFF",1.6);let a=ui(7);for(let o=0;o<140;o++)s.fillStyle="#FFF",s.beginPath(),s.arc(a()*200,a()*284,a()*.9+.5,0,7),s.fill(),a()}return{orm:Tt(t,!1),foil:Tt(e,!1)}},backOrm:()=>gl(.42,.2),back:n=>{let[t,e]=oi();Nt(e,0,0,200,284,16);let i=e.createRadialGradient(100,142,0,100,142,200);i.addColorStop(0,"#132A50"),i.addColorStop(1,"#050C1C"),e.fillStyle=i,e.fill();let s=ui(11);for(let r=0;r<220;r++)e.globalAlpha=.2+s()*.8,e.fillStyle="#E8F0FF",e.beginPath(),e.arc(s()*200,s()*284,s()*1+.2,0,7),e.fill();e.globalAlpha=1,e.strokeStyle="rgba(233,212,160,.5)",e.lineWidth=.7,e.beginPath(),e.arc(100,142,58,0,7),e.stroke(),e.beginPath(),e.arc(100,142,64,0,7),e.stroke(),e.save(),e.translate(100,142);for(let r=0;r<8;r++)e.rotate(Lt/8),e.beginPath(),e.moveTo(0,-18),e.lineTo(0,-54),e.stroke();return e.restore(),ci(e,100,142,30,Fi.gold),e.strokeStyle="rgba(233,212,160,.8)",Nt(e,7,7,186,270,10),e.stroke(),e.fillStyle="#F3E2B0",e.font=`500 13px ${Re}`,e.textAlign="center",e.fillText("Inventory",100,250),t}},steel:{art:{w:1.72,h:1.3,y:.44,r:.04,bezel:!0,rim:.03,depth:.03,photo:"duo"},edge:"#9AA3A6",edgeRough:.32,normal:"brushed",bezelCol:"#B9C1C3",face:(n,t=[],e=0)=>{let[i,s]=oi(),r=re[n];Nt(s,0,0,200,284,16);let a=s.createLinearGradient(0,0,200,284);a.addColorStop(0,"#9AA2A6"),a.addColorStop(.5,"#768084"),a.addColorStop(1,"#8E979B"),s.fillStyle=a,s.fill();let o=ui(3);for(let l=0;l<160;l++){s.strokeStyle=`rgba(${o()<.5?"255,255,255":"40,46,48"},${.02+o()*.04})`,s.lineWidth=.4;let c=o()*284;s.beginPath(),s.moveTo(0,c),s.lineTo(200,c+o()*2-1),s.stroke()}s.fillStyle=r,Nt(s,0,8,200,26,0),s.fillRect(0,8,200,26),s.fillStyle="rgba(0,0,0,.18)",s.fillRect(0,33,200,1.4),s.font=`700 12px ${Re}`,s.textBaseline="middle",s.fillStyle="#0B1416",s.fillText(ki[n],16,21);for(let l=0;l<Se[n];l++)Js(s,184-(Se[n]-1-l)*13,21,5.6,l<e?"#FFFFFF":"rgba(11,20,22,.25)");for(let[l,c]of[[12,46],[188,46],[12,272],[188,272]]){let h=s.createRadialGradient(l-1,c-1,0,l,c,4);h.addColorStop(0,"#F2F4F4"),h.addColorStop(1,"#6C7477"),s.fillStyle=h,s.beginPath(),s.arc(l,c,3.6,0,7),s.fill(),s.strokeStyle="rgba(30,34,36,.6)",s.lineWidth=.6,s.beginPath(),s.moveTo(l-2.4,c),s.lineTo(l+2.4,c),s.stroke()}{let l=s.createLinearGradient(0,184,0,276);l.addColorStop(0,"#E4E9EA"),l.addColorStop(1,"#D3DADB"),s.fillStyle=l,Nt(s,6,186,188,90,9),s.fill(),s.strokeStyle="rgba(19,25,27,.35)",s.lineWidth=.8,s.stroke()}return Pa(s,n,t,{y:206,ink:"#0B1012",sub:"#22292C",size:15,weight:600,chipBg:"rgba(19,25,27,.08)",chipStroke:"rgba(19,25,27,.35)",ptsFg:"#13191B",tagBg:"rgba(19,25,27,.12)",tagFg:"#13191B",spaceBg:"#13191B",spaceFg:"#E9EEEF"}),i},maps:n=>{let[t,e,i,s]=Ia();return i.fillStyle="rgb(0,120,115)",i.fillRect(0,0,200,284),i.fillStyle="rgb(0,150,40)",i.fillRect(0,8,200,26),i.fillStyle="rgb(0,200,0)",Nt(i,6,186,188,90,9),i.fill(),s.fillStyle="#000",s.fillRect(0,0,200,284),n>=5&&(s.fillStyle="#FFF",s.fillRect(0,8,200,26)),{orm:Tt(t,!1),foil:Tt(e,!1)}},backOrm:()=>gl(.42,.6),back:n=>{let[t,e]=oi();Nt(e,0,0,200,284,16);let i=e.createLinearGradient(0,0,200,284);i.addColorStop(0,"#8E979A"),i.addColorStop(1,"#6A7376"),e.fillStyle=i,e.fill();let s=ui(5);for(let r=0;r<240;r++){e.strokeStyle=`rgba(${s()<.5?"255,255,255":"40,46,48"},${.04+s()*.08})`,e.lineWidth=.4;let a=s()*284;e.beginPath(),e.moveTo(0,a),e.lineTo(200,a),e.stroke()}e.strokeStyle="rgba(20,26,28,.5)",e.lineWidth=1;for(let r=0;r<9;r++)e.beginPath(),e.moveTo(30+r*17.5,40),e.lineTo(30+r*17.5,244),e.stroke();return e.fillStyle="#AEB6B8",Nt(e,52,106,96,72,8),e.fill(),e.strokeStyle="rgba(20,26,28,.6)",e.stroke(),ci(e,100,136,34,Fi.teal),e.fillStyle="#1B2326",e.font=`600 13px ${Re}`,e.textAlign="center",e.fillText("Inventory",100,168),t}},paper:{art:{w:1.48,h:1.34,y:.4,r:.015,bezel:!1,z:.012,rot:-.025,mount:!0},edge:"#E3D7BC",edgeMetal:0,edgeRough:.85,sparkle:0,matte:!0,face:(n,t=[],e=0)=>{let[i,s]=oi(),r=re[n];Nt(s,0,0,200,284,14),s.fillStyle="#D9CDB3",s.fill();let a=ui(9);for(let h=0;h<500;h++)s.fillStyle=`rgba(${a()<.5?"120,96,60":"255,252,240"},${.05+a()*.12})`,s.fillRect(a()*200,a()*284,a()*1.4+.3,a()*1.4+.3);s.save(),s.translate(100,102),s.rotate(-.025),s.fillStyle="#E6DFD0",s.shadowColor="rgba(60,40,10,.35)",s.shadowBlur=6,s.shadowOffsetY=2,s.fillRect(-81,-74,162,148),s.restore(),s.font=`600 12px ${Re}`,s.textBaseline="middle",s.fillStyle="#5A4630",s.fillText(ki[n],16,21);for(let h=0;h<Se[n];h++)Js(s,184-(Se[n]-1-h)*13,21,5.6,h<e?r:"rgba(90,70,48,.2)");let o=177,l=204,c=s.createRadialGradient(o-4,l-4,1,o,l,15);c.addColorStop(0,"#FFFFFF"),c.addColorStop(.25,r),c.addColorStop(1,Qt("#000000",.55)),s.fillStyle=r,s.beginPath();for(let h=0;h<18;h++){let u=h/18*Lt,d=14+(h%2?1.5:0)*Math.sin(h*1.7);s.lineTo(o+Math.cos(u)*d,l+Math.sin(u)*d)}return s.fill(),s.fillStyle=c,s.globalAlpha=.55,s.fill(),s.globalAlpha=1,ci(s,o,l,13,Fi[Tl[n]]),Pa(s,n,t,{y:206,ink:"#2E2418",sub:"#5A4630",size:15,chipBg:"rgba(90,70,48,.0)",chipStroke:"rgba(90,70,48,.55)",ptsFg:"#2E2418",tagBg:"rgba(90,70,48,.12)",tagFg:"#2E2418",spaceBg:"#2E2418",spaceFg:"#EFE6D2"}),i},maps:n=>{let[t,e,i,s]=Ia();return i.fillStyle="rgb(0,215,0)",i.fillRect(0,0,200,284),i.fillStyle="rgb(0,90,0)",i.beginPath(),i.arc(177,204,15,0,7),i.fill(),s.fillStyle="#000",s.fillRect(0,0,200,284),{orm:Tt(t,!1),foil:Tt(e,!1)}},backOrm:()=>gl(.85,0),back:n=>{let[t,e]=oi(),i=re[n];Nt(e,0,0,200,284,14),e.fillStyle="#E9DFC8",e.fill(),e.strokeStyle="rgba(90,70,48,.5)",e.lineWidth=.8,Nt(e,10,10,180,264,8),e.stroke(),e.save(),e.translate(100,142);for(let s=0;s<14;s++)e.rotate(Lt/14),e.strokeStyle="rgba(90,70,48,.35)",e.beginPath(),e.ellipse(0,-34,9,26,0,0,7),e.stroke();return e.restore(),e.fillStyle=i,e.beginPath(),e.arc(100,142,17,0,7),e.fill(),ci(e,100,142,18,Fi[Tl[n]]),e.fillStyle="#5A4630",e.font=`500 13px ${Re}`,e.textAlign="center",e.fillText("Inventory",100,246),t}}},bu={};function gl(n,t){let e=n+","+t;if(bu[e])return bu[e];let i=Pe(4,4),s=i.getContext("2d");s.fillStyle=`rgb(0,${Math.round(n*255)},${Math.round(t*255)})`,s.fillRect(0,0,4,4);let r=Tt(i,!1);return r.keep=!0,bu[e]=r}function ui(n){return()=>{n|=0,n=n+1831565813|0;let t=Math.imul(n^n>>>15,1|n);return t=t+Math.imul(t^t>>>7,61|t)^t,((t^t>>>14)>>>0)/4294967296}}var Fa=null;function Jy(){if(Fa)return Fa;let n=Pe(256,256),t=n.getContext("2d");t.fillStyle="#808080",t.fillRect(0,0,256,256);for(let e=0;e<900;e++)t.fillStyle=`rgba(${Math.random()<.5?"255,255,255":"0,0,0"},${Math.random()*.25})`,t.fillRect(0,Math.random()*256,256,Math.random()*1.2+.3);return Fa=Ny(n,1.2),Fa.keep=!0,Fa}function _p(n,t,e){let i=Il[n]||Il.classic,s=we,r=i.face(s,t,e),a=Pe(400,568),o=a.getContext("2d");o.drawImage(r,0,0,400,568),o.save(),o.scale(2,2);let[l,c,h,u]=Zy(o,i.art);o.clip();let d=600/800,f=Math.max(h,u*d),g=f/d;return o.drawImage(un,l+(h-f)/2,c+(u-g)/2,f,g),o.restore(),i.over&&o.drawImage(i.over(s,t,e),0,0,400,568),a.toDataURL("image/png")}var qs="",Fe=100,Ky=4,ss="Rangefinder camera",xl="1970s",vl="Rangefinder camera, 1970s";function $t(n,t,e=Ky){let i=Pe(Math.round(n*Fe*e),Math.round(t*Fe*e)),s=i.getContext("2d");return s.scale(e,e),[i,s]}function qn(n,t,e,i,s,r=un){if(!r.complete||!r.naturalWidth){n.fillStyle="#6B655C",n.fillRect(t,e,i,s);return}let a=r.naturalWidth/r.naturalHeight,o=Math.max(i,s*a),l=o/a;n.drawImage(r,t+(i-o)/2,e+(s-l)/2,o,l)}var Mu={};function yl(n,t){let e=n+t;if(Mu[e])return Mu[e];let i=Pe(300,400),s=i.getContext("2d");if(!un.complete||!un.naturalWidth)return i;s.drawImage(un,0,0,300,400);let r=s.getImageData(0,0,300,400),a=r.data,o=parseInt(n.slice(1),16),l=parseInt(t.slice(1),16),c=[o>>16,o>>8&255,o&255],h=[l>>16,l>>8&255,l&255];for(let u=0;u<a.length;u+=4){let d=(a[u]*.3+a[u+1]*.59+a[u+2]*.11)/255;d=d*d*(3-2*d);for(let f=0;f<3;f++)a[u+f]=c[f]+(h[f]-c[f])*d}return s.putImageData(r,0,0),i.complete=!0,i.naturalWidth=300,i.naturalHeight=400,Mu[e]=i}function Ut(n,t,e,i,s={}){let r=s.s||12,a=()=>{n.font=`${s.i?"italic ":""}${s.w||500} ${r}px ${Re}`};if(a(),n.letterSpacing=(s.ls||0)+"px",s.mw&&n.measureText(t).width>s.mw&&(r=Math.max(r*.72,r*s.mw/n.measureText(t).width),a(),n.measureText(t).width>s.mw)){let l=t;for(;l.length>1&&n.measureText(l+"\u2026").width>s.mw;)l=l.slice(0,-1);t=l.trimEnd()+"\u2026"}n.fillStyle=s.c||"#F4F2EF",n.textAlign=s.a||"left",n.textBaseline=s.b||"alphabetic",n.fillText(t,e,i);let o=n.measureText(t).width;return n.letterSpacing="0px",o}function hn(n,t,e,i,s,r,a,o,l){let c=Se[t],h=[];for(let u=0;u<c;u++){let d=i-(c-1-u)*(l??r*2.4);Js(n,d,s,r,u<e?a:o),h.push([d,s])}return h}var li=(n,t=" \xB7 ")=>n.map(e=>e.label).join(t);function Gs(n,t,e,i,s,r,a={}){n.font=`${a.w||600} ${a.s||11}px ${Re}`,n.fillStyle=a.c||"#000",n.textAlign="center",n.textBaseline="middle";let o=a.ls??1.5,l=[...t].map(u=>n.measureText(u).width+o),c=l.reduce((u,d)=>u+d,0),h=r-c/s/2;for(let u=0;u<l.length;u++){let d=h+l[u]/2/s;n.save(),n.translate(e+Math.sin(d)*s,i-Math.cos(d)*s),n.rotate(d),n.fillText([...t][u],0,0),n.restore(),h+=l[u]/s}}function Da(n,t=3){let e=n.width,i=n.height,s=n.getContext("2d").getImageData(0,0,e,i).data,r=Pe(e,i),a=r.getContext("2d"),o=a.createImageData(e,i),l=o.data,c=(h,u)=>s[(Math.min(i-1,Math.max(0,u))*e+Math.min(e-1,Math.max(0,h)))*4]/255;for(let h=0;h<i;h++)for(let u=0;u<e;u++){let d=(c(u+1,h)-c(u-1,h))*t,f=(c(u,h+1)-c(u,h-1))*t,g=Math.hypot(d,f,1),x=(h*e+u)*4;l[x]=(-d/g*.5+.5)*255,l[x+1]=(f/g*.5+.5)*255,l[x+2]=(1/g*.5+.5)*255,l[x+3]=255}return a.putImageData(o,0,0),Tt(r,!1)}function Qy(n,t,e){let[i,s]=$t(n.w,n.h,2);return s.fillStyle=t,s.fillRect(0,0,n.w*Fe,n.h*Fe),e&&e(s),Tt(i,!1)}var rs=(n,t)=>`rgb(0,${Math.round(n*255)},${Math.round(t*255)})`;function jy(n,t){let e=new Xe(n.getPoints(t).map(i=>new j(-i.x,i.y)).reverse());return e.holes=n.holes.map(i=>new _n(i.getPoints(t).map(s=>new j(-s.x,s.y)).reverse())),e}function Wf(n,t,e,i,s=24){let r=new Dr(i?jy(n,s):n,s);i&&r.rotateY(Math.PI);let a=r.attributes.position,o=r.attributes.uv;for(let l=0;l<a.count;l++)o.setXY(l,i?.5-a.getX(l)/t:.5+a.getX(l)/t,.5+a.getY(l)/e);return r}function Fn(n,t,e,i,s,r={}){let a=r.seg??24,o=new He,l=new Ot(new ns(t,{depth:i,bevelEnabled:!1,curveSegments:a}),[new Ve({visible:!1}),s.edge]);l.position.z=-i/2;let c=new Ot(Wf(t,e.w,e.h,!1,a),s.front);c.position.z=i/2+6e-4;let h=new Ot(Wf(t,e.w,e.h,!0,a),s.back);return h.position.z=-i/2-6e-4,o.add(l,c,h),r.z&&(o.position.z=r.z),n.add(o),o}var as=(n,t,e,i=0,s=0)=>Xr(new Xe,n,t,e,i-n/2,s-t/2);function Su(n,t){let e=new Xe,i=n.length,s=r=>n[(r+i)%i];for(let r=0;r<i;r++){let[a,o]=s(r),[l,c]=s(r-1),[h,u]=s(r+1),d=Math.hypot(l-a,c-o),f=Math.hypot(h-a,u-o),g=Math.min(t,d/2,f/2),x=[a+(l-a)/d*g,o+(c-o)/d*g],m=[a+(h-a)/f*g,o+(u-o)/f*g];r===0?e.moveTo(...x):e.lineTo(...x),e.quadraticCurveTo(a,o,...m)}return e.closePath(),e}function wu(n,t,e=96){let i=new ni(n,n,t,e,1);i.rotateX(Math.PI/2);let s=i.attributes.position,r=i.attributes.uv,a=i.attributes.normal;for(let o=0;o<s.count;o++){let l=a.getZ(o);l>.5?r.setXY(o,.5+s.getX(o)/(2*n),.5+s.getY(o)/(2*n)):l<-.5&&r.setXY(o,.5-s.getX(o)/(2*n),.5+s.getY(o)/(2*n))}return i}var kt=(n,t)=>new Mn(Object.assign({roughness:.6,metalness:0,envMapIntensity:.85,emissive:n,emissiveIntensity:0},t));function qf(n,t,e,i,s,r){let a=Tt(i),o=new Ve({map:a,color:new Pt(s,s,s),transparent:!0,blending:Qn,depthWrite:!1}),l=new Ot(new ei(t,e),o);return l.position.z=r,n.add(l),{t:a,m:o,me:l}}var t_={1:"#5E5A51",3:"#1D4A63",5:"#8A5A14"};function e_(n,t,e,i,s){let r=t.toUpperCase().charCodeAt(0),a=r>=48&&r<=57,o=r*2654435761>>>0,l=[];for(let u=0;u<8&&l.length<3;u++)o&1<<(u*3+r)%8&&l.push(u);for(;l.length<2;)l.push((r+l.length*3)%8);let c=s/22,h=(u,d,f,g)=>{n.moveTo(e+u*c,i+d*c),n.lineTo(e+f*c,i+g*c)};n.beginPath(),a?(h(-3,0,-3,22),h(3,0,3,22)):h(0,0,0,22);for(let u of l)u===0&&(n.moveTo(e+3.2*c,i-1*c),n.arc(e,i-1*c,3.2*c,0,Lt)),u===1&&h(0,4,-6,-1),u===2&&h(0,4,6,-1),u===3&&h(-6,11,6,11),u===4&&(n.moveTo(e+7.6*c,i+11*c),n.arc(e+6*c,i+11*c,1.6*c,0,Lt)),u===5&&(n.moveTo(e,i+22*c),n.quadraticCurveTo(e-7*c,i+22*c,e-6*c,i+15*c)),u===6&&(n.moveTo(e,i+22*c),n.quadraticCurveTo(e+7*c,i+22*c,e+6*c,i+15*c)),u===7&&h(-5,19,5,6);n.stroke()}function Eu(n,t,e,i,s,r){let a=t.length,o=s*.62+r,l=e-(a-1)*o/2;for(let c of t)c!==" "&&e_(n,c,l,i,s),l+=o}var Wa={rune:{name:"Rune word",w:2.1,h:2.9,d:"A slab of dark stone with the find written on it in an invented rune script, one line a word, glowing in the rarity colour, the plain words carved small beneath.",lines:[["RANGEFINDER","rangefinder",62],["CAMERA","camera",128],["1970S","1970s",194]],shape(){return Su([[-1,1.32],[-.5,1.45],[.4,1.4],[1.03,1.18],[1.05,-.92],[.78,-1.43],[-.42,-1.45],[-1.03,-1.08]],.08)},stone(n,t,e,i){let s=ui(i);n.fillStyle="#34383B",n.fillRect(0,0,t,e);for(let r=0;r<2600;r++){let a=s();n.fillStyle=a<.5?`rgba(255,255,255,${(.02+s()*.05).toFixed(3)})`:`rgba(0,0,0,${(.04+s()*.08).toFixed(3)})`,n.fillRect(s()*t,s()*e,1+s()*2.2,1+s()*2.2)}},face(n,t,e){let i=this,[s,r]=$t(i.w,i.h);i.stone(r,210,290,7),r.strokeStyle="rgba(8,10,11,.85)",r.lineWidth=3.4,r.lineCap="round";for(let[a,,o]of i.lines)Eu(r,a,105,o-24,22,3.2);for(let[,a,o]of i.lines)Ut(r,a,105,o+16,{s:12,w:600,c:"#C9D0CE",a:"center",ls:1.2});return hn(r,n,e,176,30,5.6,re[n],"rgba(10,12,13,.75)"),r.fillStyle="rgba(201,208,206,.3)",r.fillRect(40,226,130,1),Ut(r,`${ki[n]} \xB7 No. 024 \xB7 +${Ge[n]}`,105,246,{s:11.5,w:600,c:"#C9D0CE",a:"center",mw:170}),t.length&&Ut(r,li(t),105,266,{s:11,c:"#9FA8A6",a:"center",mw:166}),s},glow(n){let t=this,[e,i]=$t(t.w,t.h,2);i.strokeStyle=re[n],i.lineCap="round",i.shadowColor=re[n],i.shadowBlur=8,i.lineWidth=2.2;for(let[s,,r]of t.lines)Eu(i,s,105,r-24,22,3.2);return e},back(n){let t=this,[e,i]=$t(t.w,t.h);return t.stone(i,210,290,11),i.strokeStyle="rgba(8,10,11,.8)",i.lineWidth=3,i.beginPath(),i.moveTo(105,110),i.lineTo(135,140),i.lineTo(105,190),i.lineTo(75,140),i.closePath(),i.moveTo(75,140),i.lineTo(135,140),i.stroke(),Ut(i,"No. 024",105,226,{s:12,w:600,c:"#8F9896",a:"center",ls:1.5}),e},height(n){let t=this,[e,i]=$t(t.w,t.h,2);i.fillStyle="#B4B4B4",i.fillRect(0,0,210,290),i.strokeStyle="#1A1A1A",i.lineWidth=3.4,i.lineCap="round",i.filter="blur(.8px)";for(let[s,,r]of t.lines)Eu(i,s,105,r-24,22,3.2);return i.filter="none",e},starAt:(n,t)=>[176-(t-1-n)*13.4,30],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),normalMap:Da(r.height(e),4),normalScale:new j(1.2,1.2),roughness:.82}),n.backM=kt(s,{map:Tt(r.back(e)),roughness:.85}),n.artM=kt(s,{color:"#2E3235",roughness:.9}),Fn(t,r.shape(),r,.24,{front:n.front,back:n.backM,edge:n.artM},{seg:8});let a=qf(t,r.w,r.h,r.glow(e),1.5,.121);a.m.opacity=0,n.extra=[a.m],n.tick=o=>{Math.cos(n.s.fl)*Math.cos(n.s.rx)>.3&&(n.lit=Math.min(1,(n.lit||0)+o/.7)),a.m.opacity=(n.lit||0)*(.8+.2*Math.sin($e*3))}}},polaroid:{name:"Instant photo",w:2,h:2.42,d:"An instant photo that develops in your hand: it comes out grey and the find fades in. Your name for it is written on the strip in pen.",face(n,t,e){let[i,s]=$t(this.w,this.h);s.fillStyle="#F3F1EA",s.fillRect(0,0,200,242),s.fillStyle="rgba(0,0,0,.035)";for(let r=0;r<400;r++)s.fillRect(Math.random()*200,Math.random()*242,1,1);return s.save(),s.beginPath(),s.rect(11,11,178,178),s.clip(),qn(s,11,11,178,178),s.restore(),s.strokeStyle="rgba(0,0,0,.18)",s.lineWidth=.6,s.strokeRect(11,11,178,178),s.save(),s.translate(16,212),s.rotate(-.025),Ut(s,vl,0,0,{s:15,w:500,i:!0,c:"#22336B",mw:166}),s.restore(),Ut(s,t.length?li(t,", "):`No. 024  +${Ge[n]}`,18,231,{s:10.5,i:!0,c:"#3E4F86",mw:184-(Se[n]-1)*11-14-18}),hn(s,n,e,184,231,4.6,re[n],"rgba(34,51,107,.18)"),i},back(){let[n,t]=$t(this.w,this.h);return t.fillStyle="#2C2A27",t.fillRect(0,0,200,242),t.fillStyle="#3A3733",t.fillRect(0,196,200,46),Ut(t,"Inventory instant film",100,222,{s:10.5,w:600,c:"#8D877D",a:"center",ls:1}),n},starAt:(n,t)=>[184-(t-1-n)*11,231],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),roughness:.78}),n.backM=kt(s,{map:Tt(r.back(e)),roughness:.7}),n.artM=kt(s,{color:"#ECE9E1",roughness:.85}),Fn(t,as(r.w,r.h,.03),r,.03,{front:n.front,back:n.backM,edge:n.artM});let a=new Ve({color:"#3F3A34",transparent:!0,opacity:1,depthWrite:!1}),o=new Ot(new ei(1.78,1.78),a);o.position.set(0,r.h/2-.11-.89,.017),t.add(o),n.extra=[a],n.tick=l=>{Math.cos(n.s.fl)*Math.cos(n.s.rx)>.3&&(n.dev=Math.min(1,(n.dev||0)+l/2.6));let h=n.dev||0;a.opacity=1-h*h*(3-2*h),o.visible=a.opacity>.01}}},coin:{name:"Coin",w:2.12,h:2.12,d:"A heavy coin struck with the find: its photo in relief at the centre, the name round the rim, gold, silver or bronze by rarity.",tone:n=>({5:["#5A3A0C","#F6D68A"],3:["#3C4A52","#E6EEF2"],1:["#4A3020","#D9A77A"]})[n],face(n,t,e){let[i,s]=this.tone(n),[r,a]=$t(2.12,2.12),o=a.createRadialGradient(80,70,10,106,106,110);return o.addColorStop(0,s),o.addColorStop(1,i),a.fillStyle=o,a.fillRect(0,0,212,212),a.save(),a.beginPath(),a.arc(106,110,64,0,Lt),a.clip(),qn(a,42,46,128,128,yl(i,s)),a.restore(),a.strokeStyle=Qt(i,.7),a.lineWidth=1.4,a.beginPath(),a.arc(106,106,74,0,Lt),a.stroke(),a.beginPath(),a.arc(106,106,100,0,Lt),a.stroke(),Gs(a,"RANGEFINDER CAMERA",106,106,87,0,{s:12,w:700,c:i,ls:2}),Gs(a,`1970S \xB7 ${t.length?t[0].label.toUpperCase():"NO. 024"} \xB7 +${Ge[n]}`,106,106,87,Math.PI,{s:10.5,w:700,c:i,ls:1.5}),hn(a,n,e,106+(Se[n]-1)*6,34,4.4,"#FFFFFF",Qt(i,.45),12),r},back(n){let[t,e]=this.tone(n),[i,s]=$t(2.12,2.12),r=s.createRadialGradient(130,70,10,106,106,110);return r.addColorStop(0,e),r.addColorStop(1,t),s.fillStyle=r,s.fillRect(0,0,212,212),ci(s,106,100,70,[e,e,t,Qt(t,.8),t]),Gs(s,"INVENTORY",106,106,87,0,{s:12,w:700,c:t,ls:3}),Gs(s,"NO. 024",106,106,87,Math.PI,{s:11,w:700,c:t,ls:2}),i},height(n,t){let[e,i]=$t(2.12,2.12,2);return i.fillStyle="#555",i.fillRect(0,0,212,212),i.lineWidth=7,i.strokeStyle="#999",i.beginPath(),i.arc(106,106,102,0,Lt),i.stroke(),t?(i.fillStyle="#AAA",i.beginPath(),i.moveTo(106,66),i.lineTo(141,92),i.lineTo(106,140),i.lineTo(71,92),i.closePath(),i.fill()):(i.save(),i.beginPath(),i.arc(106,110,64,0,Lt),i.clip(),i.filter="blur(1.2px) grayscale(1) contrast(.8)",i.globalAlpha=.75,qn(i,42,46,128,128),i.restore()),i.filter="none",i.globalAlpha=1,Gs(i,t?"INVENTORY":"RANGEFINDER CAMERA",106,106,87,0,{s:12,w:700,c:"#CCC",ls:t?3:2}),e},starAt:(n,t)=>[106+(t-1)*6-(t-1-n)*12,34],build(n,t,e,i,s){let r=this,a=bs[e];n.front=kt(s,{map:Tt(r.face(e,[],0)),normalMap:Da(r.height(e,!1),5),metalness:1,roughness:.32,envMapIntensity:1.1}),n.backM=kt(s,{map:Tt(r.back(e)),normalMap:Da(r.height(e,!0),5),metalness:1,roughness:.32,envMapIntensity:1.1}),n.artM=kt(s,{color:a[1],metalness:1,roughness:.38}),t.add(new Ot(wu(1.06,.16),[n.artM,n.front,n.backM]))}},stamp:{name:"Postage stamp",w:1.86,h:2.36,d:"A postage stamp with the find engraved in two inks, its points as the value, and a postmark from the space it went into.",shape(){let n=this.w,t=this.h,e=.045,i=.155,s=new Xe;s.moveTo(-n/2,-t/2);let r=(a,o,l,c)=>{let h=Math.hypot(l-a,c-o),u=Math.floor(h/i),d=(l-a)/h,f=(c-o)/h,g=(h-(u-1)*i)/2;for(let x=0;x<u;x++){let m=a+d*(g+x*i),p=o+f*(g+x*i);s.lineTo(m-d*e,p-f*e),s.absarc(m,p,e,Math.atan2(-f,-d),Math.atan2(f,d),!0)}s.lineTo(l,c)};return r(-n/2,-t/2,n/2,-t/2),r(n/2,-t/2,n/2,t/2),r(n/2,t/2,-n/2,t/2),r(-n/2,t/2,-n/2,-t/2),s},face(n,t,e){let i=re[n],[s,r]=$t(this.w,this.h);r.fillStyle="#F7F4EC",r.fillRect(0,0,186,236);let a=r.createLinearGradient(0,14,0,182);a.addColorStop(0,Qt(i,.35)),a.addColorStop(1,Qt(i,.8)),r.fillStyle=a,r.fillRect(14,14,158,168),r.save(),r.beginPath(),r.rect(22,34,142,130),r.clip(),qn(r,22,34,142,130,yl(t_[n],"#F7F4EC")),r.restore(),Ut(r,"+"+Ge[n],22,29,{s:16,w:700,c:"#FFFFFF"}),hn(r,n,e,162,23,4.6,"#FFFFFF","rgba(255,255,255,.35)"),Ut(r,"INVENTORY",93,177,{s:9.5,w:700,c:"#FFFFFF",a:"center",ls:2.5}),Ut(r,vl,93,202,{s:12.5,w:600,c:"#2A2620",a:"center",mw:166}),Ut(r,"Camera \xB7 No. 024",93,218,{s:10.5,c:"#6B6457",a:"center"});let o=t.find(l=>l.k==="space");r.save(),r.globalAlpha=.55,r.strokeStyle="#2B2F5A",r.lineWidth=1.6,r.beginPath(),r.arc(150,150,25,0,Lt),r.stroke(),r.beginPath(),r.arc(150,150,20,0,Lt),r.stroke(),Ut(r,"5 NOV",150,148,{s:8,w:700,c:"#2B2F5A",a:"center"}),Ut(r,(o?o.label:"KEPT").toUpperCase(),150,158,{s:7,w:700,c:"#2B2F5A",a:"center"});for(let l=0;l<4;l++){r.beginPath(),r.moveTo(40,132+l*8);for(let c=40;c<=124;c+=6)r.lineTo(c,132+l*8+Math.sin(c/6)*2.4);r.stroke()}return r.restore(),s},back(){let[n,t]=$t(this.w,this.h);t.fillStyle="#EEEAE0",t.fillRect(0,0,186,236),t.strokeStyle="rgba(0,0,0,.05)";for(let e=0;e<186;e+=8)t.beginPath(),t.moveTo(e,0),t.lineTo(e,236),t.stroke();return Ut(t,"Inventory \xB7 No. 024",93,122,{s:10,w:600,c:"rgba(60,54,44,.4)",a:"center"}),n},starAt:(n,t)=>[162-(t-1-n)*11,23],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),roughness:.72}),n.backM=kt(s,{map:Tt(r.back()),roughness:.35,clearcoat:.6,clearcoatRoughness:.3}),n.artM=kt(s,{color:"#F1EDE3",roughness:.9}),Fn(t,r.shape(),r,.012,{front:n.front,back:n.backM,edge:n.artM},{seg:6})}},dogtag:{name:"Dog tag",w:1.5,h:2.5,d:"A steel tag on a ball chain with the find stamped into it: name, number, rarity stars. The back says where it is kept and lists the tags.",shape(){let n=as(1.5,2.5,.34),t=new _n;return t.absarc(0,.98,.09,0,Lt,!0),n.holes.push(t),n},lines:n=>["RANGEFINDER","CAMERA","1970S","NO 024",ki[n].toUpperCase(),"+"+Ge[n]],blines:n=>["KEPT IN",(n.find(t=>t.k==="space")?.label||"CAMERAS").toUpperCase(),...n.filter(t=>t.k!=="space").slice(0,3).map(t=>t.label.toUpperCase())],stampText(n,t,e){t.forEach((i,s)=>Ut(n,i,18,66+s*23,{s:14,w:700,c:e,ls:1,mw:114}))},face(n,t,e){let[i,s]=$t(1.5,2.5),r=s.createLinearGradient(0,0,150,250);return r.addColorStop(0,"#D9DDDF"),r.addColorStop(.5,"#AEB4B8"),r.addColorStop(1,"#C9CED1"),s.fillStyle=r,s.fillRect(0,0,150,250),s.save(),s.translate(.8,.8),this.stampText(s,this.lines(n,t),"rgba(255,255,255,.55)"),s.restore(),this.stampText(s,this.lines(n,t),"#3A4044"),hn(s,n,e,128,214,5,re[n],"rgba(58,64,68,.35)",13),i},back(n,t){let[e,i]=$t(1.5,2.5);i.fillStyle="#B9BEC1",i.fillRect(0,0,150,250);let s=this.blines(t.length?t:hi(fs()));return i.save(),i.translate(.8,.8),this.stampText(i,s,"rgba(255,255,255,.5)"),i.restore(),this.stampText(i,s,"#3A4044"),e},height(n,t,e){let[i,s]=$t(1.5,2.5,2);s.fillStyle="#6E6E6E",s.fillRect(0,0,150,250);for(let r=0;r<260;r++)s.fillStyle=`rgba(${Math.random()<.5?"255,255,255":"0,0,0"},.05)`,s.fillRect(0,Math.random()*250,150,.6);return s.filter="blur(.6px)",this.stampText(s,e?this.blines(t.length?t:hi(fs())):this.lines(n,t),"#C8C8C8"),s.filter="none",i},starAt:(n,t)=>[128-(t-1-n)*13,214],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),normalMap:Da(r.height(e,[],!1),4),metalness:1,roughness:.36}),n.backM=kt(s,{map:Tt(r.back(e,[])),normalMap:Da(r.height(e,[],!0),4),metalness:1,roughness:.36}),n.artM=kt(s,{color:"#B4BABE",metalness:1,roughness:.3}),Fn(t,r.shape(),r,.05,{front:n.front,back:n.backM,edge:n.artM},{seg:20});let a=34,o=new xa(new is(.03,10,8),n.artM,a),l=new pe;for(let c=0;c<a;c++){let h=c/(a-1),u=Math.PI*(1-h),d=Math.cos(u)*.26,f=.98+Math.sin(u)*.95+(h<.5?h:1-h)*.1;l.makeTranslation(d,f,Math.sin(u)*.03),o.setMatrixAt(c,l)}t.add(o),n.redrawBack=c=>{n.backM.map.image=r.back(e,c),n.backM.map.needsUpdate=!0}}},pin:{name:"Enamel pin",w:2.4,h:2.6,d:"A gold enamel pin cut in the shape of what the find is, its photo showing through the lens, with the name on an enamel banner below.",cam(){return Su([[-1.15,-.42],[1.15,-.42],[1.15,.88],[-.15,.88],[-.15,1.18],[-.75,1.18],[-.75,.88],[-1.15,.88]],.16)},ban(){return Su([[-1,-1.25],[1,-1.25],[.9,-1],[1,-.75],[-1,-.75],[-.9,-1]],.04)},face(n,t,e){let i=re[n],[s,r]=$t(2.4,2.6);r.fillStyle="#C9A24A",r.fillRect(0,0,240,260);let a=(c,h)=>[120+c*100,130-h*100];r.fillStyle=i,Nt(r,...a(-1.08,.81),216,116,12),r.fill(),r.fillStyle=Qt("#FFFFFF",.14),Nt(r,...a(-1.08,.81),216,40,12),r.fill(),r.fillStyle="#2B2F33",Nt(r,...a(-.68,1.11),46,22,6),r.fill(),r.fillStyle="#F2E9D8",r.beginPath(),r.arc(...a(.7,.62),9,0,Lt),r.fill();let[o,l]=a(0,.23);return r.fillStyle="#F2E9D8",r.beginPath(),r.arc(o,l,52,0,Lt),r.fill(),r.fillStyle="#C9A24A",r.beginPath(),r.arc(o,l,45,0,Lt),r.fill(),r.save(),r.beginPath(),r.arc(o,l,42,0,Lt),r.clip(),qn(r,o-42,l-56,84,112),r.fillStyle="rgba(255,255,255,.18)",r.beginPath(),r.ellipse(o-14,l-16,18,10,-.6,0,Lt),r.fill(),r.restore(),r.fillStyle="#F2E9D8",r.beginPath(),r.moveTo(...a(-.94,-.79)),r.lineTo(...a(.94,-.79)),r.lineTo(...a(.86,-1)),r.lineTo(...a(.94,-1.21)),r.lineTo(...a(-.94,-1.21)),r.lineTo(...a(-.86,-1)),r.closePath(),r.fill(),Ut(r,ss,120,t.length?226:235,{s:14,w:700,c:"#2B2F33",a:"center",mw:170}),hn(r,n,e,199,103,5,"#FFF6DC",Qt("#2B2F33",.35),12.5),t.length&&Ut(r,li(t),120,243,{s:10,w:600,c:"#5A4520",a:"center",mw:166}),s},orm(n){let t=(e,i)=>[120+e*100,130-i*100];return Qy(n,rs(.3,1),e=>{e.fillStyle=rs(.22,0),Nt(e,...t(-1.06,.79),212,112,11),e.fill(),e.beginPath(),e.arc(...t(0,.23),52,0,Lt),e.fill(),e.fillStyle=rs(.3,1),e.beginPath(),e.arc(...t(0,.23),45,0,Lt),e.fill(),e.fillStyle=rs(.15,0),e.beginPath(),e.arc(...t(0,.23),42,0,Lt),e.fill(),e.fillStyle=rs(.25,0),e.fillRect(...t(-.92,-.8),184,40)})},back(){let[n,t]=$t(2.4,2.6);t.fillStyle="#B8923F",t.fillRect(0,0,240,260),t.strokeStyle="rgba(0,0,0,.08)";for(let e=0;e<60;e++)t.beginPath(),t.arc(120,100,6+e*3,0,Lt),t.stroke();return Ut(t,"Inventory",120,108,{s:13,w:700,c:"rgba(70,50,15,.55)",a:"center",ls:2}),n},starAt:(n,t)=>[199-(t-1-n)*12.5,103],build(n,t,e,i,s){let r=this,a=Tt(r.face(e,[],0)),o=r.orm(r);n.front=kt(s,{map:a,roughnessMap:o,metalnessMap:o,metalness:1,roughness:1,clearcoat:1,clearcoatRoughness:.08}),n.backM=kt(s,{map:Tt(r.back()),metalness:1,roughness:.38}),n.artM=kt(s,{color:"#D9B25A",metalness:1,roughness:.26}),Fn(t,r.cam(),r,.1,{front:n.front,back:n.backM,edge:n.artM},{seg:8}),Fn(t,r.ban(),r,.08,{front:n.front,back:n.backM,edge:n.artM},{seg:4});for(let l of[-.6,.6]){let c=new Ot(new ni(.025,.025,.22,10),n.artM);c.rotation.x=Math.PI/2,c.position.set(l,.25,-.15),t.add(c);let h=new Ot(new ni(.11,.11,.05,20),n.artM);h.rotation.x=Math.PI/2,h.position.set(l,.25,-.22),t.add(h)}}},amber:{name:"In amber",w:2,h:2.84,d:"The find caught in a drop of amber like something very old and very kept, on a brass plate engraved with its name.",face(n,t,e){let[i,s]=$t(2,2.84);s.fillStyle="#9A7A3E",s.fillRect(0,0,200,284);let r=l=>142-l*100,a=r(-1.12),o=s.createLinearGradient(0,a-31,0,a+31);return o.addColorStop(0,"#E4C987"),o.addColorStop(.5,"#B8924A"),o.addColorStop(1,"#8F6E30"),s.fillStyle=o,Nt(s,20,a-31,160,62,8),s.fill(),s.strokeStyle="rgba(70,48,12,.55)",s.lineWidth=1,Nt(s,25,a-26,150,52,6),s.stroke(),Ut(s,ss,100,a-6,{s:13.5,w:700,c:"#3A2808",a:"center"}),Ut(s,t.length?li(t):`1970s \xB7 No. 024 \xB7 +${Ge[n]}`,100,a+11,{s:10,w:600,c:"#5A4216",a:"center",mw:144}),hn(s,n,e,100+(Se[n]-1)*5.5,a+22,3.6,re[n]==="#A39E92"?"#5A4216":re[n],"rgba(58,40,8,.25)",11),i},back(){let[n,t]=$t(2,2.84);return t.fillStyle="#8F6E30",t.fillRect(0,0,200,284),Ut(t,"Inventory \xB7 No. 024",100,257,{s:10.5,w:600,c:"#3A2808",a:"center"}),n},starAt:(n,t)=>[100+(t-1)*5.5-(t-1-n)*11,276],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),metalness:1,roughness:.34}),n.backM=kt(s,{map:Tt(r.back()),metalness:1,roughness:.4});let a=kt(s,{color:"#B8924A",metalness:1,roughness:.3});Fn(t,as(1.6,.62,.08,0,-1.12),r,.08,{front:n.front,back:n.backM,edge:a});let o=new He;o.position.y=.3,t.add(o),n.artM=kt(s,{map:_s,color:"#FFD7A0",roughness:.7});let l=new Ot(new ya(.62,48),n.artM);l.scale.y=1.25,l.position.z=.02,o.add(l);let c=new Mn({color:"#E08A1E",transparent:!0,opacity:.5,roughness:.06,metalness:0,clearcoat:1,clearcoatRoughness:.04,envMapIntensity:1.3,depthWrite:!1,emissive:"#7A3A05",emissiveIntensity:.25,side:vn}),h=new Ot(new is(1,48,32),c);h.scale.set(.9,1.1,.36),h.renderOrder=3,o.add(h);let u=new Ot(new is(1,32,24),new Ve({color:"#FF9A30",transparent:!0,opacity:.16,blending:Qn,depthWrite:!1}));u.scale.set(.7,.9,.2),o.add(u),n.extra=[c,a],n.mats2=[a]}},scroll:{name:"Scroll",w:1.9,h:2.7,d:"A scroll that unrolls when it turns to you: the find drawn in sepia ink, its name lettered under it and red ink stars, sealed in wax on the back.",face(n,t,e){let[i,s]=$t(1.9,2.7),r=s.createRadialGradient(95,135,20,95,135,160);r.addColorStop(0,"#F1E5C8"),r.addColorStop(1,"#D9C59A"),s.fillStyle=r,s.fillRect(0,0,190,270);let a=ui(3);for(let o=0;o<30;o++)s.fillStyle=`rgba(120,80,30,${(.02+a()*.04).toFixed(3)})`,s.beginPath(),s.arc(a()*190,a()*270,4+a()*18,0,Lt),s.fill();return s.save(),s.beginPath(),s.ellipse(95,102,66,74,0,0,Lt),s.clip(),s.globalCompositeOperation="multiply",qn(s,29,28,132,148,yl("#3A220C","#F1E5C8")),s.restore(),s.strokeStyle="rgba(58,34,12,.55)",s.lineWidth=1,s.beginPath(),s.ellipse(95,102,68,76,0,0,Lt),s.stroke(),Ut(s,ss,95,202,{s:16,w:600,i:!0,c:"#3A220C",a:"center"}),Ut(s,`${xl} \xB7 No. 024 \xB7 +${Ge[n]}`,95,219,{s:11,i:!0,c:"#5A3A18",a:"center"}),t.length&&Ut(s,"Filed under "+li(t,", "),95,236,{s:10,i:!0,c:"#5A3A18",a:"center",mw:164}),hn(s,n,e,95+(Se[n]-1)*6.5,252,4.8,"#A8322A","rgba(168,50,42,.18)",13),i},back(){let[n,t]=$t(1.9,2.7);t.fillStyle="#D6C193",t.fillRect(0,0,190,270),t.fillStyle="rgba(120,80,30,.05)";for(let e=0;e<270;e+=5)t.fillRect(0,e,190,1);return n},starAt:(n,t)=>[95+(t-1)*6.5-(t-1-n)*13,252],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),roughness:.88}),n.backM=kt(s,{map:Tt(r.back()),roughness:.9}),n.artM=kt(s,{color:"#D9C59A",roughness:.9});let a=Fn(t,as(1.8,2.3,.01),r,.006,{front:n.front,back:n.backM,edge:n.artM}),o=kt(s,{color:"#6B4423",roughness:.55,clearcoat:.4}),l=kt(s,{color:"#C9A24A",metalness:1,roughness:.3}),c=()=>{let g=new He,x=new Ot(new ni(.075,.075,1.9,24),o);x.rotation.z=Math.PI/2,g.add(x);for(let m of[-1,1]){let p=new Ot(new is(.095,16,12),l);p.position.x=m*1,g.add(p)}return t.add(g),g},h=c(),u=c(),d=new Ot(new ni(.22,.23,.04,32),kt(s,{color:"#8E1F1A",roughness:.45,clearcoat:.5}));d.rotation.x=Math.PI/2,d.position.set(0,-.3,-.03),t.add(d),n.extra=[o,l,d.material];let f=g=>{let x=g*g*(3-2*g),m=.05+.95*x;a.scale.y=m,h.position.y=1.15*m+.07,u.position.y=-1.15*m-.07,d.visible=g>.8};n.roll=0,f(0),n.tick=g=>{Math.cos(n.s.fl)*Math.cos(n.s.rx)>.25&&n.roll<1&&(n.roll=Math.min(1,n.roll+g/.8),f(n.roll))}}},record:{name:"Record",w:2.2,h:2.9,d:"A record in its sleeve: the find is the cover, the record slides out as it turns to you, and the back of the sleeve lists the tags as tracks.",face(n,t,e){let i=re[n],[s,r]=$t(2.2,2.9),a=70;r.save(),r.beginPath(),r.rect(0,a,220,220),r.clip(),qn(r,0,a,220,220);let o=r.createLinearGradient(0,a+120,0,a+220);return o.addColorStop(0,"rgba(8,10,12,0)"),o.addColorStop(1,"rgba(8,10,12,.85)"),r.fillStyle=o,r.fillRect(0,a+120,220,100),r.restore(),r.fillStyle=i,r.fillRect(0,a,220,7),Ut(r,"INVENTORY RECORDS",12,a+24,{s:9.5,w:700,c:"#FFFFFF",ls:2}),hn(r,n,e,206,a+20,4.6,i,"rgba(255,255,255,.3)"),Ut(r,ss,12,a+186,{s:18,w:700,c:"#FFFFFF"}),Ut(r,`${xl} \xB7 No. 024 \xB7 +${Ge[n]}`,12,a+205,{s:11,w:500,c:"#E6E2DA"}),s},back(n,t){let e=re[n],[i,s]=$t(2.2,2.9),r=70;return s.fillStyle="#EDE8DD",s.fillRect(0,r,220,220),s.fillStyle=e,s.fillRect(0,r,220,7),Ut(s,ss,14,r+30,{s:14,w:700,c:"#24211B"}),(t.length?t:hi(fs())).forEach((o,l)=>{Ut(s,l<3?"A"+(l+1):"B"+(l-2),14,r+58+l*20,{s:11,w:700,c:e==="#A39E92"?"#5E5A51":e}),Ut(s,o.k==="space"?"Kept in "+o.label:o.label,40,r+58+l*20,{s:12,c:"#24211B",mw:166})}),Ut(s,`INV-024 \xB7 +${Ge[n]}`,14,r+206,{s:10,w:600,c:"#6B6457",ls:1}),i},label(n){let t=re[n],[e,i]=$t(2.04,2.04,2);i.fillStyle="#0D0E10",i.fillRect(0,0,204,204),i.strokeStyle="rgba(255,255,255,.07)",i.lineWidth=.6;for(let s=40;s<100;s+=2.2)i.beginPath(),i.arc(102,102,s,0,Lt),i.stroke();return i.fillStyle=t,i.beginPath(),i.arc(102,102,34,0,Lt),i.fill(),Ut(i,"INVENTORY",102,90,{s:7,w:700,c:"#FFFFFF",a:"center",ls:1.5}),Ut(i,"No. 024",102,122,{s:7,w:600,c:"#FFFFFF",a:"center"}),i.fillStyle="#0D0E10",i.beginPath(),i.arc(102,102,3,0,Lt),i.fill(),e},starAt:(n,t)=>[206-(t-1-n)*11,90],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),roughness:.62}),n.backM=kt(s,{map:Tt(r.back(e,[])),roughness:.7}),n.artM=kt(s,{color:"#D8D2C4",roughness:.8}),Fn(t,as(2.2,2.2,.02,0,-.35),r,.03,{front:n.front,back:n.backM,edge:n.artM});let a=Tt(r.label(e)),o=kt(s,{map:a,roughness:.25,clearcoat:1,clearcoatRoughness:.15}),l=kt(s,{color:"#0D0E10",roughness:.3}),c=new Ot(wu(1.02,.012),[l,o,o]);c.position.set(0,-.35,0),t.add(c),n.extra=[o,l],n.slide=0,n.tick=h=>{Math.cos(n.s.fl)*Math.cos(n.s.rx)>.3&&n.slide<1&&(n.slide=Math.min(1,n.slide+h/.9));let d=n.slide,f=1-Math.pow(1-d,3);c.position.y=-.35+.78*f,c.rotation.z=-f*2.4},n.redrawBack=h=>{n.backM.map.image=r.back(e,h),n.backM.map.needsUpdate=!0}}},book:{name:"Book",w:1.9,h:2.62,d:"A small clothbound book: the find tipped in on the cover, the title stamped in gold, the space and tags as the blurb on the back.",face(n,t,e){let i=re[n],s=bs[n],[r,a]=$t(1.9,2.62);a.fillStyle=n>=5?"#2B2433":n>=3?"#1F3442":"#3B3A36",a.fillRect(0,0,190,262),a.fillStyle="rgba(255,255,255,.035)";for(let o=0;o<262;o+=2)a.fillRect(0,o,190,.7);return a.strokeStyle=s[3],a.lineWidth=1.2,a.strokeRect(10,10,170,242),a.fillStyle="#F4F0E6",a.fillRect(36,36,118,118),a.save(),a.beginPath(),a.rect(41,41,108,108),a.clip(),qn(a,41,41,108,108),a.restore(),Ut(a,ss,95,186,{s:17,w:700,c:s[3],a:"center"}),Ut(a,xl,95,206,{s:13,w:600,c:s[3],a:"center"}),hn(a,n,e,95+(Se[n]-1)*6.5,226,4.8,s[3],"rgba(255,255,255,.12)",13),Ut(a,"Inventory \xB7 No. 024",95,244,{s:9.5,w:600,c:Qt(s[3],.7),a:"center",ls:1}),r},gold(n){let[t,e]=$t(1.9,2.62,2);e.fillStyle=rs(.85,0),e.fillRect(0,0,190,262),e.strokeStyle=rs(.35,.7),e.lineWidth=1.4,e.strokeRect(10,10,170,242);let i=rs(.35,.7);return Ut(e,ss,95,186,{s:17,w:700,c:i,a:"center"}),Ut(e,xl,95,206,{s:13,w:600,c:i,a:"center"}),Tt(t,!1)},back(n,t){let[e,i]=$t(1.9,2.62);i.fillStyle=n>=5?"#2B2433":n>=3?"#1F3442":"#3B3A36",i.fillRect(0,0,190,262);let s=t.length?t:hi(fs()),r=s.find(a=>a.k==="space");Ut(i,"Kept in "+(r?r.label:"Cameras")+".",20,60,{s:14,w:600,c:"#E8E2D4"}),s.filter(a=>a.k!=="space").slice(0,5).forEach((a,o)=>Ut(i,"\u2014 "+a.label,20,88+o*20,{s:12.5,i:!0,c:"#CFC8B8",mw:150})),Ut(i,`+${Ge[n]} points`,20,200,{s:12,w:600,c:re[n]});for(let a=0;a<28;a++)i.fillStyle="#E8E2D4",i.fillRect(120+a*2,214,a%3?1:1.6,26);return e},spine(n){let t=bs[n],[e,i]=$t(.38,2.62);return i.fillStyle=n>=5?"#241E2B":n>=3?"#1A2C38":"#33322E",i.fillRect(0,0,38,262),i.save(),i.translate(19,131),i.rotate(Math.PI/2),Ut(i,vl,0,0,{s:11.5,w:700,c:t[3],a:"center",b:"middle"}),i.restore(),e},pages(n){let[t,e]=$t(n?.38:1.9,n?1.9:.38,2);e.fillStyle="#EFE8D6",e.fillRect(0,0,400,400),e.fillStyle="rgba(120,100,70,.18)";for(let i=0;i<38;i+=1.6)n?e.fillRect(i,0,.5,400):e.fillRect(0,i,400,.5);return t},starAt:(n,t)=>[95+(t-1)*6.5-(t-1-n)*13-0,226],build(n,t,e,i,s){let r=this,a=r.gold(e);n.front=kt(s,{map:Tt(r.face(e,[],0)),roughnessMap:a,metalnessMap:a,metalness:1,roughness:1}),n.backM=kt(s,{map:Tt(r.back(e,[])),roughness:.85}),n.artM=kt(s,{map:Tt(r.spine(e)),roughness:.85});let o=kt(s,{map:Tt(r.pages(!0)),roughness:.9}),l=kt(s,{map:Tt(r.pages(!1)),roughness:.9});t.add(new Ot(new Vn(1.9,2.62,.38),[o,n.artM,l,l,n.front,n.backM])),n.extra=[o,l],n.redrawBack=c=>{n.backM.map.image=r.back(e,c),n.backM.map.needsUpdate=!0}}},tape:{name:"Cassette",w:2.6,h:1.64,d:"A cassette with the find written on its label in marker, the reels turning in the window. Side B carries the tags.",label(n,t,e,i){let s=re[n],[r,a]=$t(2.6,1.64);a.fillStyle="#23272A",a.fillRect(0,0,260,164),a.fillStyle="#F2EEE4",Nt(a,16,12,228,112,6),a.fill(),a.fillStyle=s,a.fillRect(16,30,228,6),Ut(a,i,26,27,{s:12,w:700,c:"#23272A"}),Ut(a,i==="A"?vl:li(t.length?t:hi(fs())),44,27,{s:12.5,w:600,i:!0,c:"#22336B",mw:192}),a.fillStyle="#2E3336",Nt(a,70,52,120,46,23),a.fill(),a.fillStyle="#5A3A22";for(let o of[100,160])a.beginPath(),a.arc(o,75,16,0,Lt),a.fill();a.fillStyle="#F2EEE4";for(let o of[100,160])a.beginPath(),a.arc(o,75,7,0,Lt),a.fill();i==="A"&&(Ut(a,`No. 024 \xB7 +${Ge[n]}`,26,116,{s:10.5,w:600,c:"#6B6457"}),hn(a,n,e,234,112,4.4,s,"rgba(35,39,42,.2)")),a.fillStyle="#1B1E20",a.beginPath(),a.moveTo(60,164),a.lineTo(76,134),a.lineTo(184,134),a.lineTo(200,164),a.fill();for(let o of[8,252])a.fillStyle="#3A3F42",a.beginPath(),a.arc(o,8,3,0,Lt),a.fill();return r},starAt:(n,t)=>[234-(t-1-n)*10.6,112],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.label(e,[],0,"A")),roughness:.55}),n.backM=kt(s,{map:Tt(r.label(e,[],0,"B")),roughness:.55}),n.artM=kt(s,{color:"#23272A",roughness:.45}),t.add(new Ot(new Vn(2.6,1.64,.22),[n.artM,n.artM,n.artM,n.artM,n.front,n.backM]));let a=kt(s,{color:"#F2EEE4",roughness:.5}),o=[-.3,.3].map(l=>{let c=new Ot(new ni(.06,.06,.24,6),a);return c.rotation.x=Math.PI/2,c.position.set(l,.07,0),t.add(c),c});n.extra=[a],n.tick=l=>{for(let c of o)c.rotation.y+=l*2.4},n.redrawBack=l=>{n.backM.map.image=r.label(e,l,0,"B"),n.backM.map.needsUpdate=!0}}},neon:{name:"Neon sign",w:2.5,h:1.9,d:"A neon sign of the find\u2019s name that flickers on as it turns to you, on a dark metal plate. The stars light up one tube at a time.",plate(n){let[t,e]=$t(2.5,1.9);e.fillStyle="#181C1E",e.fillRect(0,0,250,190),e.fillStyle="rgba(255,255,255,.025)";for(let i=0;i<190;i+=1.5)e.fillRect(0,i,250,.6);for(let[i,s]of[[12,12],[238,12],[12,178],[238,178]])e.fillStyle="#3B4246",e.beginPath(),e.arc(i,s,4,0,Lt),e.fill();return e.strokeStyle="rgba(200,210,214,.18)",e.lineWidth=2.4,e.lineCap="round",this.tubes(e,n,5,!0),t},tubes(n,t,e,i){n.font=`italic 700 34px ${Re}`,n.textAlign="center",n.textBaseline="alphabetic",n.lineJoin="round",n.strokeText("Rangefinder",125,84),n.font=`italic 700 30px ${Re}`,n.strokeText("camera",125,124),n.font=`600 13px ${Re}`,n.strokeText(`1970s \xB7 No. 024 \xB7 +${Ge[t]}`,125,160),i||hn(n,t,e,125+(Se[t]-1)*8,36,7,n.strokeStyle,"rgba(0,0,0,0)",16)},glow(n,t){let e=re[n],[i,s]=$t(2.5,1.9,2),r=new Pt(e).lerp(new Pt("#FFFFFF"),.55).getStyle();s.lineCap="round",s.shadowColor=e,s.shadowBlur=14,s.strokeStyle=e,s.lineWidth=4.2,this.tubes(s,n,0,!1),s.shadowBlur=4,s.strokeStyle=r,s.lineWidth=1.6,this.tubes(s,n,0,!1),s.shadowBlur=10,s.shadowColor=e;for(let a=0;a<Math.min(t,Se[n]);a++)Js(s,125+(Se[n]-1)*8-(Se[n]-1-a)*16,36,7,r);return i},back(n,t){let[e,i]=$t(2.5,1.9);return i.fillStyle="#202528",i.fillRect(0,0,250,190),i.fillStyle="#E9E4D8",Nt(i,30,120,190,44,4),i.fill(),Ut(i,"Inventory signs \xB7 No. 024",40,138,{s:10.5,w:600,c:"#3A3F42"}),Ut(i,li(t.length?t:hi(fs())),40,155,{s:10.5,c:"#3A3F42",mw:172}),e},starAt:(n,t)=>[125+(t-1)*8-(t-1-n)*16,36],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.plate(e)),roughness:.55,metalness:.4}),n.backM=kt(s,{map:Tt(r.back(e,[])),roughness:.6}),n.artM=kt(s,{color:"#15191B",roughness:.5,metalness:.5}),Fn(t,as(2.5,1.9,.06),r,.08,{front:n.front,back:n.backM,edge:n.artM});let a=new Ot(new Vn(.7,.32,.16),n.artM);a.position.set(.5,.45,-.12),t.add(a);let o=qf(t,2.5,1.9,r.glow(e,0),1.35,.07);n.extra=[o.m],n.glowT=o.t,o.m.opacity=0,n.tick=l=>{Math.cos(n.s.fl)*Math.cos(n.s.rx)>.3&&(n.on=(n.on||0)+l);let h=n.on||0;o.m.opacity=h<=0?0:h<.5?Math.sin(h*90)>.2?.9:.15:.94+.06*Math.sin($e*40)*(Math.random()<.02?1:0)},n.redrawGlow=l=>{o.t.image=r.glow(e,l),o.t.needsUpdate=!0}}},voxel:{name:"Pixel item",w:2,h:2.84,d:"The find rebuilt out of little blocks like a game item, floating over its item box: name in the rarity colour, stars and points.",COLS:20,ROWS:22,tip(n,t,e){let i=re[n],[s,r]=$t(2,2.84),a=191;return r.fillStyle="#0E1416",Nt(r,6,a,188,72,8),r.fill(),r.strokeStyle=i,r.lineWidth=2,Nt(r,7,a+1,186,70,7),r.stroke(),Ut(r,ss,16,a+22,{s:14.5,w:700,c:i==="#A39E92"?"#E8E4DA":i}),Ut(r,t.length?li(t):"Camera \xB7 1970s \xB7 No. 024",16,a+40,{s:10.5,c:"#B8C4C2",mw:168}),Ut(r,`+${Ge[n]}`,16,a+60,{s:12,w:700,c:"#E2A23B"}),hn(r,n,e,182,a+56,4.6,i,"rgba(255,255,255,.15)"),s},starAt:(n,t)=>[182-(t-1-n)*11,247],build(n,t,e,i,s){let r=this,a=r.COLS*r.ROWS,o=Tt(r.tip(e,[],0));n.front=kt(s,{map:o,roughness:.5}),n.backM=kt(s,{color:"#0E1416",roughness:.5});let l=kt(s,{color:i,roughness:.4,metalness:.3});Fn(t,as(1.88,.72,.08,0,-.85),r,.06,{front:n.front,back:n.backM,edge:l}),n.artM=new bn({roughness:.55,metalness:0,emissive:s,emissiveIntensity:0});let c=new xa(new Vn(.078,.078,.078),n.artM,a),h=new pe,u=new Pt,d=Pe(r.COLS,r.ROWS),f=d.getContext("2d");un.complete&&un.naturalWidth&&f.drawImage(un,0,0,r.COLS,r.ROWS);let g=f.getImageData(0,0,r.COLS,r.ROWS).data;for(let x=0,m=0;x<r.ROWS;x++)for(let p=0;p<r.COLS;p++,m++){let T=g[m*4]/255,M=g[m*4+1]/255,y=g[m*4+2]/255,D=T*.3+M*.59+y*.11;h.makeTranslation((p-(r.COLS-1)/2)*.086,.42+((r.ROWS-1)/2-x)*.086,(D-.4)*.12),c.setMatrixAt(m,h),u.setRGB(T,M,y,Ke),c.setColorAt(m,u)}t.add(c),n.extra=[l],n.tick=x=>{c.position.y=Math.sin($e*2)*.025}}},film:{name:"Film strip",w:1.3,h:2.96,d:"A strip of 35 mm film with the find on the middle frame, the frames before and after it dimmer, edge print down the side and a tape label for the tags.",shape(){let n=as(1.3,2.96,.02);for(let t=0;t<9;t++)for(let e of[-.56,.56])n.holes.push(Xr(new _n,.085,.12,.02,e-.0425,-1.36+t*.34-.06));return n},face(n,t,e){let i=re[n],[s,r]=$t(1.3,2.96);r.fillStyle="#3A2414",r.fillRect(0,0,130,296);let a=(o,l)=>{r.save(),r.beginPath(),r.rect(17,o-46,96,92),r.clip(),qn(r,17,o-60+(l?0:6),96,120),l&&(r.fillStyle=`rgba(20,12,6,${l})`,r.fillRect(17,o-46,96,92)),r.restore()};return a(48,.55),a(148,0),a(248,.55),r.save(),r.translate(9,148),r.rotate(-Math.PI/2),Ut(r,"INVENTORY 400   23   \u25B824   25",0,3,{s:6.5,w:700,c:"#E9A23B",a:"center",b:"middle",ls:1}),r.restore(),r.save(),r.translate(121,148),r.rotate(Math.PI/2),Ut(r,"RANGEFINDER CAMERA 1970S \xB7 No. 024",0,3,{s:6.5,w:700,c:"#E9A23B",a:"center",b:"middle",ls:1}),r.restore(),r.fillStyle="#F2EEE4",r.save(),r.translate(65,262),r.rotate(-.04),Nt(r,-46,-11,92,22,2),r.fill(),Ut(r,t.length?li(t):`+${Ge[n]} \xB7 ${ki[n]}`,0,4,{s:8.5,w:600,i:!0,c:"#22336B",a:"center",mw:86}),r.restore(),hn(r,n,e,65+(Se[n]-1)*6,92,4,i,"rgba(233,162,59,.25)",12),s},back(){let[n,t]=$t(1.3,2.96);t.fillStyle="#2A1A0E",t.fillRect(0,0,130,296),t.fillStyle="rgba(0,0,0,.25)";for(let e of[48,148,248])t.fillRect(17,e-46,96,92);return n},starAt:(n,t)=>[65+(t-1)*6-(t-1-n)*12,92],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),roughness:.3,clearcoat:.7,clearcoatRoughness:.2}),n.backM=kt(s,{map:Tt(r.back()),roughness:.55}),n.artM=kt(s,{color:"#3A2414",roughness:.4}),Fn(t,r.shape(),r,.01,{front:n.front,back:n.backM,edge:n.artM},{seg:4})}},astro:{name:"Star disc",w:2.2,h:2.2,d:"A brass star disc: the find in a window of night sky at its centre, its name engraved round the scale, and an open-work rete of star pointers turning over it.",face(n,t,e){let i=re[n],[s,r]=$t(2.2,2.2),a=r.createRadialGradient(80,70,10,110,110,115);a.addColorStop(0,"#E9C877"),a.addColorStop(1,"#8A6424"),r.fillStyle=a,r.fillRect(0,0,220,220),r.save(),r.beginPath(),r.arc(110,110,58,0,Lt),r.clip(),r.fillStyle="#0B1B33",r.fillRect(0,0,220,220),r.globalAlpha=.9,r.globalCompositeOperation="screen",qn(r,52,42,116,140,yl("#0B1B33","#CFE0FF")),r.globalCompositeOperation="source-over",r.globalAlpha=1;let o=ui(5);r.fillStyle="#FFFFFF";for(let l=0;l<60;l++)r.globalAlpha=.3+o()*.6,r.fillRect(52+o()*116,52+o()*116,1,1);r.restore(),r.globalAlpha=1,r.strokeStyle="rgba(60,40,10,.75)",r.lineWidth=1;for(let l of[60,72,98,104])r.beginPath(),r.arc(110,110,l,0,Lt),r.stroke();for(let l=0;l<72;l++){let c=l/72*Lt,h=l%6?4:7;r.beginPath(),r.moveTo(110+Math.sin(c)*104,110-Math.cos(c)*104),r.lineTo(110+Math.sin(c)*(104-h),110-Math.cos(c)*(104-h)),r.stroke()}return Gs(r,"RANGEFINDER CAMERA \xB7 1970S",110,110,85,0,{s:11,w:700,c:"#3A2608",ls:1.6}),Gs(r,t.length?li(t.slice(0,3)).toUpperCase():`NO. 024 \xB7 +${Ge[n]}`,110,110,85,Math.PI,{s:9.5,w:700,c:"#3A2608",ls:1.2}),hn(r,n,e,110+(Se[n]-1)*6,66,3.8,i==="#A39E92"?"#FFFFFF":i,"rgba(255,255,255,.25)",12),s},back(){let[n,t]=$t(2.2,2.2);t.fillStyle="#9C7430",t.fillRect(0,0,220,220);let e=ui(9),i=[];for(let s=0;s<40;s++){let r=e()*Lt,a=e()*92;i.push([110+Math.sin(r)*a,110-Math.cos(r)*a])}t.strokeStyle="rgba(50,32,8,.55)",t.lineWidth=.8;for(let s=1;s<12;s++)t.beginPath(),t.moveTo(...i[s-1]),t.lineTo(...i[s]),t.stroke();t.fillStyle="rgba(50,32,8,.7)";for(let s of i)t.beginPath(),t.arc(s[0],s[1],1.6,0,Lt),t.fill();return Ut(t,"Inventory \xB7 No. 024",110,200,{s:10,w:700,c:"rgba(50,32,8,.75)",a:"center"}),n},starAt:(n,t)=>[110+(t-1)*6-(t-1-n)*12,66],build(n,t,e,i,s){let r=this;n.front=kt(s,{map:Tt(r.face(e,[],0)),metalness:.85,roughness:.38}),n.backM=kt(s,{map:Tt(r.back()),metalness:1,roughness:.4}),n.artM=kt(s,{color:"#B88A3A",metalness:1,roughness:.3}),t.add(new Ot(wu(1.1,.07),[n.artM,n.front,n.backM]));let a=new Xe;a.absarc(0,0,1.08,0,Lt,!1);let o=new _n;o.absarc(0,0,1,0,Lt,!0),a.holes.push(o);let l=new Xe;l.absarc(.12,.18,.52,0,Lt,!1);let c=new _n;c.absarc(.12,.18,.47,0,Lt,!0),l.holes.push(c);let h=new He,u=kt(s,{color:bs[5][1],metalness:1,roughness:.22});h.add(new Ot(new ns([a,l],{depth:.025,bevelEnabled:!1,curveSegments:48}),u));for(let d=0;d<5;d++){let f=d/5*Lt+.3,g=new Xe;g.moveTo(-.04,0),g.quadraticCurveTo(0,.1,.02,.34),g.quadraticCurveTo(.03,.1,.04,0),g.closePath();let x=new Ot(new ns(g,{depth:.02,bevelEnabled:!1}),u);x.position.set(Math.sin(f)*.66,Math.cos(f)*.66,0),x.rotation.z=-f+Math.PI,h.add(x)}h.position.z=.04,t.add(h),n.extra=[u],n.tick=d=>{h.rotation.z+=d*.12}}}},e2=Object.keys(Wa),ku=n=>Math.min(1,2.15/n.w,2.95/n.h),n_=n=>n.shape?[n.shape()]:n.cam?[n.cam(),n.ban()]:null,bp=(n,t)=>n.round?`border-radius:50%;box-shadow:0 0 22px ${Qt(t,.5)}, 0 10px 24px rgba(0,0,0,.35)`:`filter:drop-shadow(0 0 12px ${Qt(t,.55)}) drop-shadow(0 8px 12px rgba(0,0,0,.35))`;function i_(n,t,e={}){let i=we,s=re[i],r={r:i,C:s,look:"find:"+t,find:n,x:e.x??180,y:e.y??360,z:e.z??0,tags:[],stars:0,s:{rx:0,ry:0,vrx:0,vry:0,fl:e.up?0:Math.PI,vfl:0,rz:0,vrz:0,sc:e.sc??1},flipT:e.up?0:Math.PI,tx:0,ty:0,k:120,d:11,kf:150,df:14,idle:!1,ptr:null,interactive:!1,jit:0,heat:0,glowA:0,glowCol:new Pt(s),sheen:-1,sheenA:0,vis:1},a=new He,o=new He,l=new He;a.add(o),o.add(l),r.root=a,r.body=o,r.key=t;let c=ku(n);l.scale.setScalar(c),r.fit=c;let h=new Pt("#FFFFFF").lerp(new Pt(s),.45);n.build(r,l,i,s,h),r.sparkU={uSheen:{value:-1},uSheenA:{value:0}},r.glowM=new Ve({map:up,color:r.glowCol,transparent:!0,opacity:0,blending:Qn,depthWrite:!1});let u=new Ot(new ei(n.w*c*1.5,n.h*c*1.35),r.glowM);return u.position.z=-.3,a.add(u),r.glow=u,r.mats=[r.front,r.backM,r.artM,...r.extra||[]],r.faceCv=n.face?n.face(i,[],0):null,r.redraw=()=>{if(n.face){let d=n.face(i,r.tags,r.stars);r.faceCv=d,r.front.map.image=d,r.front.map.needsUpdate=!0}else if(n.label){let d=n.label(i,r.tags,r.stars,"A");r.faceCv=d,r.front.map.image=d,r.front.map.needsUpdate=!0}else if(n.tip){let d=n.tip(i,r.tags,r.stars);r.faceCv=d,r.front.map.image=d,r.front.map.needsUpdate=!0}r.redrawBack&&r.tags.length&&r.redrawBack(r.tags),r.redrawGlow&&r.redrawGlow(r.stars)},r.starXY=(d,f)=>{let[g,x]=n.starAt(d,f);return[100+(g-n.w*50)*c,142+(x-n.h*50)*c]},Ss.add(a),ws.add(r),r.root.renderOrder=2,r}function Mp(n,t,e){let i=Wa[n],s=we,r;if(n==="neon"){[r]=$t(i.w,i.h,2);let o=r.getContext("2d");o.drawImage(i.plate(s),0,0,i.w*Fe,i.h*Fe),o.globalCompositeOperation="lighter",o.drawImage(i.glow(s,e),0,0,i.w*Fe,i.h*Fe)}else if(n==="rune"){[r]=$t(i.w,i.h,2);let o=r.getContext("2d");o.drawImage(i.face(s,t,e),0,0,i.w*Fe,i.h*Fe),o.globalCompositeOperation="lighter",o.drawImage(i.glow(s),0,0,i.w*Fe,i.h*Fe)}else if(n==="voxel"){[r]=$t(i.w,i.h,2);let o=r.getContext("2d");o.imageSmoothingEnabled=!1;let l=Pe(i.COLS,i.ROWS);un.complete&&l.getContext("2d").drawImage(un,0,0,i.COLS,i.ROWS),o.drawImage(l,100-i.COLS*4.3,100-i.ROWS*4.3,i.COLS*8.6,i.ROWS*8.6),o.imageSmoothingEnabled=!0,o.drawImage(i.tip(s,t,e),0,0,i.w*Fe,i.h*Fe)}else if(n==="amber"){[r]=$t(i.w,i.h,2);let o=r.getContext("2d");o.save(),o.beginPath(),o.ellipse(100,112,90,110,0,0,Lt),o.clip(),qn(o,10,2,180,220),o.fillStyle="rgba(224,138,30,.45)",o.fillRect(0,0,200,230),o.restore();let l=i.face(s,t,e);o.save(),o.beginPath(),o.rect(20,222,160,62),o.clip(),o.drawImage(l,0,0,200,284),o.restore()}else if(n==="tape")r=i.label(s,t,e,"A");else if(n==="record"){[r]=$t(i.w,i.h,2);let o=r.getContext("2d");o.save(),o.beginPath(),o.arc(110,102,102,0,Lt),o.clip(),o.drawImage(i.label(s),8,0,204,204),o.restore(),o.drawImage(i.face(s,t,e),0,0,i.w*Fe,i.h*Fe)}else r=i.face(s,t,e);let a=n_(i);if(a){let[o,l]=$t(i.w,i.h,2),c=h=>[i.w*50+h.x*Fe,i.h*50-h.y*Fe];l.beginPath();for(let h of a)for(let u of[h,...h.holes])u.getPoints(24).forEach((d,f)=>f?l.lineTo(...c(d)):l.moveTo(...c(d))),l.closePath();l.clip("evenodd"),l.drawImage(r,0,0,i.w*Fe,i.h*Fe),r=o}return{src:r.toDataURL("image/png"),w:i.w*Fe,h:i.h*Fe,round:n==="coin"||n==="astro"}}document.head.appendChild(We(`<style>
.tagp.bk { box-sizing: border-box; overflow: hidden; }
.tagp.bk .ted { display: flex; flex-direction: column; flex: 1; min-height: 0; }
.tagp.bk .cd { height: 3px; border-radius: 2px; background: rgba(255,255,255,.1); position: relative; margin-top: 8px; flex: none; }
.tagp.bk .cd i { position: absolute; left: 0; top: 0; bottom: 0; width: 100%; border-radius: 2px; }
.tagp.bk .pick { margin-top: 6px; }

.tagp.postcard { background: #F3EEE3; color: #2A2620; padding: 12px; box-shadow: inset 0 0 0 1px rgba(0,0,0,.06); }
.tagp.postcard .pc { display: grid; grid-template-columns: 1fr 1px 1fr; gap: 10px; flex: 1; min-height: 0; }
.tagp.postcard .pc > b { background: rgba(42,38,32,.25); }
.tagp.postcard .note { background: repeating-linear-gradient(180deg, transparent 0 21px, rgba(42,38,32,.18) 21px 22px); box-shadow: none; color: #2A2620; font-style: italic; font-size: 14px; line-height: 22px; padding: 2px 2px; height: 100%; border-radius: 0; }
.tagp.postcard .note::placeholder { color: rgba(42,38,32,.45); }
.tagp.postcard .stamps { display: flex; flex-wrap: wrap; gap: 5px; justify-content: flex-end; }
.tagp.postcard .tg { height: auto; padding: 5px 2px 5px 7px; border-radius: 2px; background: #FFFDF8; color: #2A2620; font-size: 11.5px; box-shadow: 0 0 0 2px #FFFDF8, 0 0 0 3px rgba(42,38,32,.25), 0 2px 4px rgba(0,0,0,.15); outline: 2.5px dotted #F3EEE3; outline-offset: -1px; }
.tagp.postcard .tg:nth-child(3n+1) { border-left: 3px solid #3E8DB5; } .tagp.postcard .tg:nth-child(3n+2) { border-left: 3px solid #B5705A; } .tagp.postcard .tg:nth-child(3n) { border-left: 3px solid #4F8A5B; }
.tagp.postcard .tg .x { width: 22px; height: 22px; color: #6B6457; font-size: 15px; }
.tagp.postcard .add { height: 30px; padding: 0 10px; border-color: rgba(42,38,32,.4); color: #2A2620; font-size: 12.5px; border-radius: 3px; }
.tagp.postcard .addr { margin-top: auto; }
.tagp.postcard .to { font-size: 12px; color: #6B6457; display: block; margin-bottom: 4px; }
.tagp.postcard .spc { background: transparent; color: #2A2620; height: 30px; padding: 0 4px; border-bottom: 1.5px solid rgba(42,38,32,.4); border-radius: 0; font-size: 16px; }
.tagp.postcard .pk { background: #FFFDF8; color: #2A2620; box-shadow: inset 0 0 0 1px rgba(42,38,32,.25); height: 28px; font-size: 12px; }
.tagp.postcard .pcr { display: flex; flex-direction: column; gap: 8px; min-height: 0; }
.tagp.postcard .cd { background: rgba(42,38,32,.12); }
.pmark { position: absolute; right: 18px; top: 10px; width: 120px; height: 70px; pointer-events: none; opacity: 0; }

.tagp.chart { background: radial-gradient(120% 90% at 50% 30%, #13254A 0%, #0A1430 55%, #050A1C 100%); color: #E8ECF7; padding: 14px; box-shadow: inset 0 0 0 1px rgba(217,183,106,.45), inset 0 0 0 4px rgba(5,10,28,1), inset 0 0 0 5px rgba(217,183,106,.25); }
.tagp.chart::before { content: ''; position: absolute; inset: 0; background-image: radial-gradient(1px 1px at 20px 30px, #fff, transparent), radial-gradient(1px 1px at 120px 80px, #cfe, transparent), radial-gradient(1.4px 1.4px at 220px 50px, #fff, transparent), radial-gradient(1px 1px at 70px 200px, #fff, transparent), radial-gradient(1px 1px at 180px 260px, #fde, transparent), radial-gradient(1px 1px at 250px 330px, #fff, transparent), radial-gradient(1.2px 1.2px at 40px 340px, #fff, transparent); opacity: .7; pointer-events: none; }
.tagp.chart .ct { font-size: 12px; color: #D9B76A; text-align: center; }
.tagp.chart .spc { display: flex; margin: 2px auto 0; background: transparent; color: #F4EBD2; font-size: 21px; font-weight: 500; height: 34px; }
.tagp.chart .pick { justify-content: center; }
.tagp.chart .pk { background: rgba(217,183,106,.1); color: #F4EBD2; box-shadow: inset 0 0 0 1px rgba(217,183,106,.35); height: 30px; font-size: 13px; }
.tagp.chart .sky { position: relative; flex: 1; min-height: 150px; }
.tagp.chart .sky svg { position: absolute; inset: 0; width: 100%; height: 100%; overflow: visible; }
.tagp.chart .tg { position: absolute; transform: translate(-9px, -9px); height: auto; padding: 0; background: transparent; box-shadow: none; display: flex; align-items: center; gap: 4px; font-size: 12.5px; color: #E8ECF7; }
.tagp.chart .tg .sd { width: 18px; height: 18px; border-radius: 50%; background: radial-gradient(circle, #fff 0 2.5px, rgba(255,255,255,.5) 3.5px, transparent 9px); flex: none; }
.tagp.chart .tg .x { width: 24px; height: 24px; color: rgba(232,236,247,.55); font-size: 15px; }
.tagp.chart .add { align-self: center; height: 32px; border-color: rgba(217,183,106,.5); color: #F4EBD2; font-size: 13px; }
.tagp.chart .note { background: rgba(255,255,255,.05); box-shadow: inset 0 0 0 1px rgba(217,183,106,.25); color: #F4EBD2; font-size: 13.5px; margin-top: 8px; }
.tagp.chart .cd { background: rgba(217,183,106,.15); }

.tagp.tip { background: linear-gradient(180deg, #14161A, #0B0C0F); color: #DADCE0; padding: 14px 14px 12px; box-shadow: inset 0 0 0 1px var(--rc), inset 0 0 0 3px #0B0C0F, inset 0 0 0 4px rgba(255,255,255,.08), 0 0 30px -6px var(--rc); }
.tagp.tip .nm { font-size: 17px; font-weight: 600; color: var(--rc); line-height: 1.2; }
.tagp.tip .ty { font-size: 13px; color: #8E929A; margin: 3px 0 10px; display: flex; align-items: center; gap: 4px; flex-wrap: wrap; }
.tagp.tip .spc { height: 28px; padding: 0 8px; background: rgba(255,255,255,.06); color: #F1F2F4; font-size: 13px; font-weight: 500; border-radius: 6px; }
.tagp.tip .pk { height: 28px; border-radius: 6px; font-size: 12.5px; background: rgba(255,255,255,.05); color: #DADCE0; box-shadow: inset 0 0 0 1px rgba(255,255,255,.12); }
.tagp.tip hr { border: 0; height: 1px; background: linear-gradient(90deg, transparent, rgba(255,255,255,.18), transparent); margin: 8px 0; width: 100%; }
.tagp.tip .ats { display: flex; flex-direction: column; gap: 1px; }
.tagp.tip .tg { height: 30px; background: transparent; box-shadow: none; padding: 0 0 0 2px; color: #8FB8FF; font-size: 14.5px; justify-content: space-between; border-radius: 6px; }
.tagp.tip .tg:hover { background: rgba(143,184,255,.06); }
.tagp.tip .tg .x { width: 26px; height: 26px; color: #6E737C; font-size: 16px; }
.tagp.tip .add { height: 30px; border: 0; padding: 0 2px; text-align: left; color: #6E737C; font-size: 14px; }
.tagp.tip .note { background: transparent; box-shadow: none; color: #D9B76A; font-style: italic; font-size: 14px; padding: 2px; }
.tagp.tip .note::placeholder { color: rgba(217,183,106,.5); }
.tagp.tip .cd { background: rgba(255,255,255,.06); }

.tagp.stick { background: #B9925E; background-image: repeating-linear-gradient(17deg, rgba(255,255,255,.04) 0 2px, transparent 2px 5px), repeating-linear-gradient(-23deg, rgba(0,0,0,.04) 0 1px, transparent 1px 4px); padding: 14px; color: #2A1E10; }
.tagp.stick .badge { width: 92px; height: 92px; border-radius: 50%; margin: 0 auto 8px; display: grid; place-items: center; text-align: center; box-shadow: 0 0 0 4px #fff, 0 3px 8px rgba(0,0,0,.3); transform: rotate(-6deg); }
.tagp.stick .badge .spc { background: transparent; color: #fff; font-size: 15px; font-weight: 700; flex-direction: column; height: auto; padding: 0; gap: 2px; text-shadow: 0 1px 2px rgba(0,0,0,.3); }
.tagp.stick .badge .spc i { display: none; }
.tagp.stick .pick { justify-content: center; }
.tagp.stick .pk { background: #fff; color: #2A1E10; box-shadow: 0 2px 4px rgba(0,0,0,.2); height: 30px; font-size: 12.5px; }
.tagp.stick .row { gap: 8px; justify-content: center; }
.tagp.stick .tg { height: 34px; background: var(--sk); color: #fff; font-weight: 600; font-size: 13.5px; box-shadow: 0 0 0 3px #fff, 0 3px 6px rgba(0,0,0,.28); transform: rotate(var(--rot)); padding-left: 12px; }
.tagp.stick .tg .x { color: rgba(255,255,255,.85); width: 28px; height: 28px; }
.tagp.stick .add { height: 34px; border: 2px dashed rgba(255,255,255,.75); color: #fff; border-radius: 10px; font-weight: 600; }
.tagp.stick .note { margin-top: 12px; background: #FFF2A8; box-shadow: 0 3px 6px rgba(0,0,0,.25); color: #3A2E10; border-radius: 2px; transform: rotate(1.2deg); font-size: 14px; }
.tagp.stick .note::placeholder { color: rgba(58,46,16,.5); }
.tagp.stick .cd { background: rgba(0,0,0,.15); margin-top: auto; }

.tagp.pass { background: #EEE7D6; background-image: repeating-radial-gradient(circle at 50% 120%, rgba(62,141,181,.06) 0 2px, transparent 2px 7px); color: #2D2A22; padding: 14px; box-shadow: inset 0 0 0 1px rgba(0,0,0,.08); }
.tagp.pass .ph { display: flex; justify-content: space-between; font-size: 12px; color: #7A7262; margin-bottom: 10px; }
.tagp.pass .visa { align-self: flex-start; transform: rotate(-4deg); border: 2.5px solid var(--ink); box-shadow: inset 0 0 0 2px #EEE7D6, inset 0 0 0 3.5px var(--ink); padding: 8px 12px; border-radius: 4px; color: var(--ink); margin-bottom: 14px; }
.tagp.pass .visa small { display: block; font-size: 10.5px; opacity: .8; }
.tagp.pass .spc { background: transparent; color: var(--ink); font-size: 17px; font-weight: 700; height: 26px; padding: 0; }
.tagp.pass .spc i { display: none; }
.tagp.pass .pk { background: transparent; color: #2D2A22; box-shadow: inset 0 0 0 1.5px rgba(45,42,34,.35); height: 28px; font-size: 12.5px; border-radius: 4px; }
.tagp.pass .row { gap: 10px; }
.tagp.pass .tg { width: 84px; height: 84px; border-radius: 50%; padding: 0; justify-content: center; flex-direction: column; background: transparent; color: var(--ink); box-shadow: inset 0 0 0 2px var(--ink), inset 0 0 0 5px #EEE7D6, inset 0 0 0 6px var(--ink); font-size: 12px; font-weight: 600; text-align: center; transform: rotate(var(--rot)); white-space: normal; line-height: 1.15; position: relative; opacity: .9; }
.tagp.pass .tg .x { position: absolute; top: -4px; right: -4px; width: 24px; height: 24px; background: #EEE7D6; color: #7A7262; font-size: 15px; }
.tagp.pass .add { width: 84px; height: 84px; border-radius: 50%; border: 2px dashed rgba(45,42,34,.35); color: #7A7262; font-size: 13px; padding: 0; }
.tagp.pass .note { margin-top: 10px; background: repeating-linear-gradient(180deg, transparent 0 21px, rgba(45,42,34,.2) 21px 22px); box-shadow: none; color: #2D2A22; line-height: 22px; font-size: 14px; border-radius: 0; padding: 0 2px; }
.tagp.pass .cd { background: rgba(45,42,34,.1); margin-top: auto; }

.tagp.tkt { background: #F6F1E6; color: #24211B; padding: 0; box-shadow: inset 0 0 0 1px rgba(0,0,0,.06); }
.tagp.tkt .tb { background: var(--rc); color: #fff; padding: 10px 14px; font-size: 14px; font-weight: 600; display: flex; justify-content: space-between; }
.tagp.tkt .tb small { font-weight: 400; opacity: .85; }
.tagp.tkt .tm { padding: 12px 14px 8px; display: flex; flex-direction: column; gap: 4px; flex: 1; min-height: 0; }
.tagp.tkt .tg { height: 30px; background: transparent; box-shadow: none; padding: 0; border-bottom: 1px dashed rgba(36,33,27,.2); border-radius: 0; justify-content: space-between; font-size: 14.5px; color: #24211B; }
.tagp.tkt .tg .x { width: 26px; height: 26px; color: #8A8273; font-size: 16px; }
.tagp.tkt .add { height: 30px; border: 0; padding: 0; text-align: left; color: #8A8273; font-size: 14px; }
.tagp.tkt .note { margin-top: 6px; background: rgba(36,33,27,.04); box-shadow: none; color: #24211B; font-size: 13.5px; }
.tagp.tkt .stub { position: relative; border-top: 2px dashed rgba(36,33,27,.3); padding: 10px 14px 12px; display: flex; align-items: center; justify-content: space-between; background: #F6F1E6; }
.tagp.tkt .stub::before, .tagp.tkt .stub::after { content: ''; position: absolute; top: -9px; width: 16px; height: 16px; border-radius: 50%; background: #0A0F10; }
.tagp.tkt .stub::before { left: -8px; } .tagp.tkt .stub::after { right: -8px; }
.tagp.tkt .stub small { font-size: 11.5px; color: #8A8273; display: block; }
.tagp.tkt .spc { background: transparent; color: #24211B; font-size: 16px; height: 28px; padding: 0; }
.tagp.tkt .pk { background: rgba(36,33,27,.06); color: #24211B; box-shadow: none; height: 28px; font-size: 12.5px; }
.tagp.tkt .cd { margin: 0 14px 0; background: rgba(36,33,27,.1); }

.tagp.dial { background: radial-gradient(120% 80% at 50% 25%, #3A4044, #1C2023 70%); color: #E6E9EB; padding: 14px; box-shadow: inset 0 0 0 1px rgba(255,255,255,.1); }
.tagp.dial .dl { position: relative; width: 210px; height: 190px; margin: 0 auto; flex: none; }
.tagp.dial .knob { position: absolute; left: 55px; top: 45px; width: 100px; height: 100px; border-radius: 50%; background: conic-gradient(from 0deg, #8E979B, #D7DDE0, #6F777B, #C9CFD2, #7B8387, #D7DDE0, #8E979B); box-shadow: 0 6px 14px rgba(0,0,0,.5), inset 0 0 0 6px rgba(0,0,0,.12); }
.tagp.dial .knob::after { content: ''; position: absolute; left: 47px; top: 6px; width: 6px; height: 26px; border-radius: 3px; background: var(--rc); box-shadow: 0 0 10px var(--rc); }
.tagp.dial .knob::before { content: ''; position: absolute; inset: 22px; border-radius: 50%; background: radial-gradient(circle at 35% 30%, #4A5256, #23282B); }
.tagp.dial .pk { position: absolute; transform: translate(-50%, -50%); height: 28px; padding: 0 10px; font-size: 12.5px; background: rgba(255,255,255,.04); color: #AEB5B9; box-shadow: inset 0 0 0 1px rgba(255,255,255,.1); }
.tagp.dial .pk.on { color: #fff; background: rgba(255,255,255,.12); box-shadow: inset 0 0 0 1.5px var(--rc); }
.tagp.dial .lbl { text-align: center; color: #8E979B; }
.tagp.dial .row { justify-content: center; }
.tagp.dial .tg { background: rgba(255,255,255,.07); box-shadow: inset 0 0 0 1px rgba(255,255,255,.14); color: #E6E9EB; }
.tagp.dial .add { border-color: rgba(255,255,255,.3); color: #C9CFD2; }
.tagp.dial .cd { margin-top: auto; }

.tagp.eng { background: linear-gradient(135deg, #A9B0B3, #7E878B 50%, #9AA2A6); background-image: repeating-linear-gradient(0deg, rgba(255,255,255,.05) 0 1px, rgba(0,0,0,.03) 1px 2px), linear-gradient(135deg, #A9B0B3, #7E878B 50%, #9AA2A6); color: #22282B; padding: 16px; box-shadow: inset 0 0 0 1px rgba(255,255,255,.35); }
.tagp.eng .lbl { color: #3A4246; text-shadow: 0 1px 0 rgba(255,255,255,.35); }
.tagp.eng .spc, .tagp.eng .tg, .tagp.eng .pk { background: rgba(0,0,0,.08); color: #1B2023; box-shadow: inset 0 2px 3px rgba(0,0,0,.35), inset 0 -1px 0 rgba(255,255,255,.35); text-shadow: 0 1px 0 rgba(255,255,255,.4); font-weight: 600; border-radius: 6px; }
.tagp.eng .spc i, .tagp.eng .pk i { box-shadow: inset 0 1px 2px rgba(0,0,0,.5); }
.tagp.eng .tg .x { color: #3A4246; }
.tagp.eng .add { border: 1.5px dashed rgba(27,32,35,.45); color: #2A3236; border-radius: 6px; }
.tagp.eng .note { background: rgba(0,0,0,.08); box-shadow: inset 0 2px 3px rgba(0,0,0,.3); color: #1B2023; border-radius: 6px; }
.tagp.eng .field { background: rgba(0,0,0,.1); color: #1B2023; box-shadow: inset 0 2px 3px rgba(0,0,0,.3); }
.tagp.eng .cd { background: rgba(0,0,0,.15); margin-top: auto; }

.tagp.small { background: rgba(8,20,22,.9); padding: 10px; color: #E8ECEB; }
.tagp.small .lbl { display: none; }
.tagp.small .row { gap: 5px; }
.tagp.small .spc, .tagp.small .pk { height: 30px; font-size: 12.5px; padding: 0 8px 0 10px; }
.tagp.small .tg { height: 30px; font-size: 12.5px; padding-left: 10px; }
.tagp.small .tg .x { width: 26px; height: 26px; font-size: 16px; }
.tagp.small .add { height: 30px; font-size: 12.5px; padding: 0 10px; }
.tagp.small .row + .row { margin-top: 6px; }
.tagp.small .cd { margin-top: auto; }

.tagp.holo { background: linear-gradient(180deg, rgba(120,240,255,.12), rgba(120,240,255,.05)); background-image: repeating-linear-gradient(0deg, rgba(160,250,255,.07) 0 1px, transparent 1px 3px), linear-gradient(180deg, rgba(120,240,255,.14), rgba(120,240,255,.04)); color: #CFFBFF; padding: 16px; box-shadow: inset 0 0 0 1px rgba(150,245,255,.55), 0 0 24px rgba(120,240,255,.25); text-shadow: 1px 0 rgba(255,80,160,.45), -1px 0 rgba(80,200,255,.6); }
.tagp.holo .hh { font-size: 12px; color: #8FEFFF; margin-bottom: 4px; }
.tagp.holo .lbl { color: #7FDDEB; }
.tagp.holo .spc, .tagp.holo .pk, .tagp.holo .tg { background: rgba(140,240,255,.1); color: #E6FDFF; box-shadow: inset 0 0 0 1px rgba(150,245,255,.5); border-radius: 4px; }
.tagp.holo .tg .x { color: #9FF3FF; }
.tagp.holo .add { border: 1px dashed rgba(150,245,255,.6); color: #BFF8FF; border-radius: 4px; }
.tagp.holo .note { background: rgba(140,240,255,.06); box-shadow: inset 0 0 0 1px rgba(150,245,255,.35); color: #E6FDFF; border-radius: 4px; }
.tagp.holo .cd { background: rgba(150,245,255,.15); margin-top: auto; }
.hbeam { position: absolute; pointer-events: none; opacity: 0; background: linear-gradient(0deg, rgba(140,240,255,.45), rgba(140,240,255,0)); clip-path: polygon(42% 100%, 58% 100%, 100% 0, 0 0); filter: blur(2px); mix-blend-mode: screen; }

.tagp.lib { background: #E8D7A4; background-image: linear-gradient(180deg, transparent 0 46px, rgba(181,60,60,.7) 46px 48px, transparent 48px), repeating-linear-gradient(180deg, transparent 0 33px, rgba(62,100,160,.28) 33px 34px); color: #2A2416; padding: 12px 14px; }
.tagp.lib .lh { display: flex; justify-content: space-between; align-items: baseline; height: 30px; }
.tagp.lib .lh small { font-size: 11.5px; color: #6E6346; }
.tagp.lib .spc { background: transparent; color: #2A2416; height: 30px; padding: 0 2px; font-size: 15px; font-weight: 600; }
.tagp.lib .pk { background: rgba(255,255,255,.35); color: #2A2416; box-shadow: none; height: 26px; font-size: 12px; }
.tagp.lib .rows { margin-top: 12px; display: flex; flex-direction: column; }
.tagp.lib .tg { height: 34px; background: transparent; box-shadow: none; border-radius: 0; padding: 0; justify-content: flex-start; gap: 10px; color: #2A2416; font-size: 14.5px; }
.tagp.lib .tg .dt { color: #6A3FA0; font-size: 11.5px; font-weight: 700; border: 1.5px solid #6A3FA0; border-radius: 3px; padding: 1px 4px; transform: rotate(-3deg); opacity: .8; flex: none; }
.tagp.lib .tg .x { margin-left: auto; width: 26px; height: 26px; color: #6E6346; font-size: 16px; }
.tagp.lib .add { height: 34px; border: 0; padding: 0; text-align: left; color: #6E6346; font-size: 14px; }
.tagp.lib .note { background: transparent; box-shadow: none; color: #2A2416; font-size: 14px; line-height: 34px; padding: 0 2px; border-radius: 0; }
.tagp.lib .note::placeholder { color: rgba(42,36,22,.45); }
.tagp.lib .cd { background: rgba(42,36,22,.12); margin-top: auto; }
.kept { position: absolute; left: 50%; top: 52%; transform: translate(-50%, -50%) rotate(-12deg); color: #6A3FA0; border: 3px solid #6A3FA0; border-radius: 6px; padding: 4px 14px; font-size: 26px; font-weight: 700; opacity: 0; pointer-events: none; mix-blend-mode: multiply; }

.tagp.min { background: rgba(9,13,15,.88); padding: 16px; color: #F4F2EF; }
.tagp.min .lbl { display: none; }
.tagp.min .row { gap: 8px; }
.tagp.min .row + .row { margin-top: 14px; }
.tagp.min .spc { height: 48px; padding: 0 14px 0 16px; border-radius: 24px; font-size: 17px; }
.tagp.min .spc i { width: 12px; height: 12px; }
.tagp.min .pk { height: 44px; border-radius: 22px; font-size: 15px; }
.tagp.min .tg { height: 44px; border-radius: 22px; font-size: 16px; padding-left: 16px; background: rgba(244,242,239,.1); box-shadow: none; }
.tagp.min .tg .x { width: 40px; height: 40px; font-size: 22px; }
.tagp.min .add { width: 44px; height: 44px; padding: 0; border-radius: 22px; border: 0; background: #F4F2EF; color: #0E333A; font-size: 24px; line-height: 1; }
.tagp.min .cd { margin-top: auto; }
</style>`));var s_=(n,t,e)=>{let i=(e?284:200)*n,s=(e?200:284)*n;return{l:180-i/2,t:t-s/2,w:i,h:s,r:16*n}},Ri=(n,t={})=>n.ts.tags.map((e,i)=>`<span class="tg"${t.style?` style="${t.style(e,i)}"`:""}>${t.pre?t.pre(e,i):""}${t.hash===!1?"":"# "}${e}<button type="button" class="x" data-t="${e}" aria-label="Remove ${e}">\xD7</button></span>`).join(""),os=(n,t,e="row pick")=>`<div class="${e}"${t?"":" hidden"}>${dn.filter(i=>i!==n.ts.space).map(i=>`<button type="button" class="pk" data-s="${i.label}"><i style="background:${i.col}"></i>${i.label}</button>`).join("")}</div>`,ls=n=>`<button type="button" class="spc"><i style="background:${n.ts.space.col}"></i>${n.ts.space.label}${pp}</button>`,Ws=(n,t=2,e="Why you kept it")=>`<textarea class="field note" rows="${t}" placeholder="${e}">${n.ts.note}</textarea>`,cs=n=>[...n.querySelectorAll(".tg")].pop(),hs=(n,t,e)=>It(n,t,e).finished;function pn(n){return{name:n.name,d:n.d,key:n.key,run(t,e){return Na(t,i=>{let s=n.sc??1.45,r=n.y??352,a=n.box||s_(s,r,n.land),o=n.pad??11;e.interactive=!1,e.idle=!1,e.tx=0,e.ty=0,e.k=120,e.d=18,e.ptr=null;let l=d=>{n.flip==="x"?e.tx=d?Math.PI:0:Iu(e,e.flipT+Math.PI,110,17),e.rzT=d&&n.land?-Math.PI/2:0};l(!0),De(e,{y:n.cardY??r},460,ee.inOut),De(e.s,{sc:n.cardSc??s},460,ee.inOut),Z.play("whoosh");let c=di(`<div class="tagp bk ${n.skin}" style="position:absolute;left:${a.l+o}px;top:${a.t+o}px;width:${a.w-2*o}px;height:${a.h-2*o}px;border-radius:${Math.max(6,a.r-o*.7)}px;pointer-events:auto;display:flex;flex-direction:column;opacity:0;--rc:${e.C}">${n.head?n.head(e):""}</div>`),h=n.editor?n.editor(e,i):Ua(e,i,{note:n.note??!0,noteRows:n.noteRows??3,tagLabel:n.tagLabel,tpl:n.tpl,after:n.after,fx:n.fx?n.fx(c,e):null});c.appendChild(h);let u=h._left;if(!n.timer){let d=We(`<div class="cd"><i style="background:${n.cdCol||e.C}"></i></div>`);(n.cdIn&&c.querySelector(n.cdIn)||c).appendChild(d),u=d.firstElementChild,mp(c,h)}return n.enter?n.enter(c,e,t):It(c,[{opacity:0,transform:"scale(.97)"},{opacity:1,transform:"none"}],220,{delay:430}),{tick:d=>{u.style.width=100-d*100+"%"},onHold:()=>{u.style.opacity="0"},leave:async()=>{if(n.leave)try{await n.leave(c,e,t)}catch{}It(c,[{opacity:getComputedStyle(c).opacity},{opacity:0}],140),await ke(120,t),l(!1),De(e,{y:360},460,ee.inOut),De(e.s,{sc:1},460,ee.inOut),Z.play("whoosh"),await ke(520,t),e.k=70,e.d=9,e.idle=!0,e.interactive=!0,e.rzT=0},cleanup:()=>{c.remove(),n.cleanup?.()}}})}}}var Xf=["#3E6FB5","#8A3F6A","#2F7A5B","#B5562F","#5A4FB5"],r_=["#E2574C","#3E8DB5","#4F8A5B","#8A6FB5","#E2A23B"];Xa.push(pn({key:"C1",name:"Postcard",skin:"postcard",land:!0,sc:1.2,y:360,d:"The card turns over and lies on its side as a postcard: your note on the left, the space as the address, the tags as stamps. When it goes, a postmark lands on the stamps.",tpl:(n,t)=>`<div class="pc"><div class="pcl">${Ws(n,6,"Why you kept it\u2026")}</div><b></b><div class="pcr"><div class="stamps">${Ri(n,{hash:!1})}<button type="button" class="add">+ Stamp</button></div><div class="addr"><span class="to">To</span>${ls(n)}${os(n,t)}</div></div></div>`,fx:n=>({add:t=>{let e=cs(t);e&&It(e,[{transform:"translateY(-30px) rotate(-12deg)",opacity:0},{transform:"none",opacity:1}],260,{ease:"cubic-bezier(.3,1.4,.5,1)"}),Z.play("stamp")}}),leave:async(n,t,e)=>{let i=We(`<svg class="pmark" viewBox="0 0 120 70"><circle cx="34" cy="35" r="27" fill="none" stroke="#3A3F7A" stroke-width="2.4"/><circle cx="34" cy="35" r="21" fill="none" stroke="#3A3F7A" stroke-width="1"/><text x="34" y="33" text-anchor="middle" font-size="9" fill="#3A3F7A" font-family="Google Sans Flex">5 Nov</text><text x="34" y="44" text-anchor="middle" font-size="7.5" fill="#3A3F7A" font-family="Google Sans Flex">Inventory</text>${[0,1,2,3].map(s=>`<path d="M62 ${20+s*10} q8 -5 14 0 t14 0 t14 0 t14 0" fill="none" stroke="#3A3F7A" stroke-width="2"/>`).join("")}</svg>`);n.appendChild(i),Z.play("stamp"),Ys(.05),await It(i,[{opacity:0,transform:"scale(1.5) rotate(-14deg)"},{opacity:.85,transform:"scale(1) rotate(-8deg)"}],160,{ease:"cubic-bezier(.5,0,.8,.5)"}).finished,await ke(380,e)}}),pn({key:"C2",name:"Star chart",skin:"chart",d:"The back is a patch of night sky. The space is the constellation\u2019s name, every tag is a star joined to the last, Add places a new star and \xD7 sends one falling. The note is a wish.",tpl:(n,t)=>{let e=[[22,30],[112,12],[150,66],[96,118],[24,134],[60,80],[150,150],[16,88]],i=n.ts.tags.map((s,r)=>e[r%e.length]);return`<div class="ct">Constellation</div>${ls(n)}${os(n,t)}<div class="sky"><svg>${i.slice(1).map((s,r)=>`<line x1="${i[r][0]}" y1="${i[r][1]}" x2="${s[0]}" y2="${s[1]}" stroke="rgba(217,183,106,.55)" stroke-width="1"/>`).join("")}</svg>${Ri(n,{hash:!1,style:(s,r)=>`left:${i[r][0]}px;top:${i[r][1]}px`,pre:()=>'<i class="sd"></i>'})}</div><button type="button" class="add">+ Add a star</button>${Ws(n,2,"Make a wish")}`},fx:()=>({add:n=>{let t=cs(n);if(t){It(t,[{opacity:0,transform:"translate(-9px,-9px) scale(2.4)"},{opacity:1,transform:"translate(-9px,-9px) scale(1)"}],380);let[e,i]=Ba(t.querySelector(".sd"));Dn(e,i,"#FFF3C8",14,120,{g:0,life:.5})}Z.play("ting",2)},remove:n=>(Z.play("swoop"),hs(n,[{opacity:1,transform:"translate(-9px,-9px)"},{opacity:0,transform:"translate(60px,50px) scale(.4)"}],360))})}),pn({key:"C3",name:"Item card",skin:"tip",sc:1.4,y:344,d:"The back reads like an item in a role-playing game: the find\u2019s name in its rarity colour, the space as its type, each tag a line of its own, the note as flavour text in gold.",tpl:(n,t)=>`<div class="nm">Rangefinder camera, 1970s</div><div class="ty">${ki[n.r]} find, kept in ${ls(n)}</div>${os(n,t)}<hr><div class="ats">${Ri(n,{hash:!1,pre:()=>"+ "})}<button type="button" class="add">+ Add a line</button></div><hr>${Ws(n,3,"\u201CWhy you kept it.\u201D")}`,fx:()=>({add:n=>{let t=cs(n);t&&It(t,[{background:"rgba(143,184,255,.35)"},{background:"rgba(143,184,255,0)"}],600),Z.play("ting",1)},remove:n=>(Z.play("tick",700),hs(n,[{opacity:1,height:"30px"},{opacity:0,height:"0px"}],200))})}),pn({key:"C4",name:"Stickers",skin:"stick",d:"The back is kraft paper. The space is a round badge, the tags are stickers stuck on at an angle; \xD7 peels one off, Add slaps a new one on. The note is a yellow sticky note.",tpl:(n,t)=>`<div class="badge" style="background:${n.ts.space.col}">${ls(n)}</div>${os(n,t)}<div class="row">${Ri(n,{style:(e,i)=>`--sk:${r_[(e.length+i)%5]};--rot:${i*37%11-5}deg`})}<button type="button" class="add">+ Sticker</button></div>${Ws(n,3,"Why you kept it")}`,fx:()=>({add:n=>{let t=cs(n);t&&It(t,[{transform:"rotate(var(--rot)) scale(1.35)",opacity:0},{transform:"rotate(var(--rot)) scale(.94)",opacity:1,offset:.7},{transform:"rotate(var(--rot)) scale(1)"}],260,{ease:"ease-out"}),Z.play("pop")},remove:n=>(Z.play("tear"),hs(n,[{transform:"rotate(var(--rot))",opacity:1},{transform:"translate(10px,-30px) rotate(28deg) rotateX(60deg)",opacity:0}],320)),space:n=>{let t=n.querySelector(".badge");t&&It(t,[{transform:"rotate(-6deg) scale(1.3)"},{transform:"rotate(-6deg) scale(1)"}],260),Z.play("pop")}})}),pn({key:"C5",name:"Passport",skin:"pass",flip:"x",d:"The card flips up like a passport page. The space is the visa, every tag an entry stamp in its own ink; Add thumps a new stamp down. Remarks below for the note.",head:()=>'<div class="ph"><span>Entries</span><span>No. 024</span></div>',tpl:(n,t)=>`<div class="visa" style="--ink:${Xf[0]}"><small>Visa \xB7 kept in</small>${ls(n)}</div>${os(n,t)}<div class="row">${Ri(n,{hash:!1,style:(e,i)=>`--ink:${Xf[(i+1)%5]};--rot:${i*53%21-10}deg`})}<button type="button" class="add">+ Stamp</button></div>${Ws(n,2,"Remarks")}`,fx:()=>({add:n=>{let t=cs(n);t&&It(t,[{transform:"rotate(var(--rot)) scale(1.6)",opacity:0},{transform:"rotate(var(--rot)) scale(1)",opacity:.9}],150,{ease:"cubic-bezier(.6,0,.9,.5)"}),Z.play("stamp"),Ys(.06)},remove:n=>(Z.play("paper"),hs(n,[{opacity:.9},{opacity:0,filter:"blur(3px)"}],260)),space:n=>{let t=n.querySelector(".visa");t&&It(t,[{transform:"rotate(-4deg) scale(1.4)",opacity:0},{transform:"rotate(-4deg) scale(1)",opacity:1}],150),Z.play("stamp")}})}),pn({key:"C6",name:"Ticket",skin:"tkt",d:"The back is a ticket in the rarity colour: tags printed on it, the note below, the space on the stub. When it goes, the stub tears off along the perforation.",head:n=>'<div class="tb"><span>Admit one find</span><small>No. 024</small></div>',tpl:(n,t)=>`<div class="tm">${Ri(n)}<button type="button" class="add">+ Add tag</button>${Ws(n,2)}</div><div class="stub"><div><small>Space</small>${ls(n)}${os(n,t)}</div><svg width="40" height="40" viewBox="0 0 24 24"><path d="M12 2l7.5 8L12 22 4.5 10z" fill="${n.C}"/><path d="M12 2l7.5 8h-15z" fill="#fff" opacity=".4"/></svg></div>`,cdIn:".tm",fx:()=>({remove:n=>(Z.play("tick",800),hs(n,[{opacity:1,transform:"none"},{opacity:0,transform:"translateX(-20px)"}],180))}),leave:async(n,t,e)=>{let i=n.querySelector(".stub");i&&(Z.play("tear"),await It(i,[{transform:"none",opacity:1},{transform:"translate(8px, 40px) rotate(7deg)",opacity:0}],360,{ease:"cubic-bezier(.5,0,.8,.4)"}).finished,await ke(120,e))}}),(()=>{let n=null,t=[[105,18],[196,95],[105,176],[14,95]],e=[0,90,180,270];return pn({key:"C7",name:"Dial",skin:"dial",note:!1,d:"The back is a metal dial with your spaces round it; the find\u2019s space is the one it points at. Tap another and the dial clicks round to it. Tags sit under the dial.",tpl:(i,s)=>{let r=dn.indexOf(i.ts.space);return`<div class="lbl">Space</div><div class="dl"><div class="knob" style="transform:rotate(${n??e[r]}deg)"></div>${dn.map((a,o)=>`<button type="button" class="pk${o===r?" on":""}" data-s="${a.label}" style="left:${t[o][0]}px;top:${t[o][1]}px"><i style="background:${a.col}"></i>${a.label}</button>`).join("")}</div><div class="lbl">Tags</div><div class="row">${Ri(i)}<button type="button" class="add">+ Add tag</button></div>`},after:(i,s)=>{let r=dn.indexOf(s.ts.space),a=e[r],o=i.querySelector(".knob");if(n!=null&&n!==a){let l=((a-n)%360+540)%360-180,c=n;It(o,[{transform:`rotate(${c}deg)`},{transform:`rotate(${c+l}deg)`}],380,{ease:"cubic-bezier(.3,1.3,.5,1)"});for(let h=0;h<4;h++)Zs(()=>Z.play("tick",1800-h*150),h*70);n=c+l}else n=a},fx:()=>({}),cleanup:()=>{n=null}})})(),pn({key:"C8",name:"Engraved",skin:"eng",d:"The back is brushed steel and everything on it is engraved. Add burns a new tag into the metal letter by letter with sparks; \xD7 grinds one off.",fx:()=>({add:n=>{let t=cs(n);if(!t)return;It(t,[{clipPath:"inset(0 100% 0 0)"},{clipPath:"inset(0 0 0 0)"}],520,{ease:"linear"}),Z.play("zap",.5);let[e,i]=Ba(t);for(let s=0;s<8;s++)Zs(()=>Dn(e-40+s*11,i,"#FFD38A",6,120,{g:300,life:.4}),s*60)},remove:n=>{Z.play("zap",.3);let[t,e]=Ba(n);return Dn(t,e,"#FFD38A",26,200,{g:300,life:.5}),hs(n,[{opacity:1,transform:"none"},{opacity:.6,transform:"translateX(-2px)",offset:.3},{opacity:0,transform:"translateX(2px)"}],260)}})}),pn({key:"C9",name:"Small back",skin:"small",sc:1,y:372,pad:9,note:!1,d:"The card turns over where it is, without growing: the back is a compact editor and the page around it stays in view. Less of a moment, more of a glance."}),pn({key:"C10",name:"Hologram",skin:"holo",cardY:600,cardSc:.62,box:{l:26,t:92,w:308,h:400,r:8},pad:0,d:"The card turns over and drops low, and its back projects a hologram above it: the space, tags and note float in the light. \xD7 dissolves a tag, Add flickers one in.",head:()=>'<div class="hh">Projected from the back of the card</div>',enter:(n,t)=>{let e=di('<div class="hbeam" style="left:60px;top:470px;width:240px;height:120px"></div>');n._beam=e,It(e,[{opacity:0},{opacity:1}],300,{delay:380}),It(n,[{opacity:0,transform:"scaleY(.02)"},{opacity:.6,transform:"scaleY(1.04)",offset:.6},{opacity:.3,offset:.75},{opacity:1,transform:"none"}],420,{delay:480}),Z.play("holo",.6)},fx:()=>({add:n=>{let t=cs(n);t&&It(t,[{opacity:0},{opacity:1,offset:.2},{opacity:.2,offset:.4},{opacity:1}],300),Z.play("holo",.25)},remove:n=>(Z.play("zap",.2),hs(n,[{opacity:1,transform:"none",filter:"none"},{opacity:0,transform:"scaleY(.1) scaleX(1.3)",filter:"blur(2px)"}],240))}),leave:async n=>{n._beam&&(It(n._beam,[{opacity:1},{opacity:0}],200),Zs(()=>n._beam.remove(),220)),await It(n,[{opacity:1,transform:"none"},{opacity:0,transform:"scaleY(.02)"}],200).finished}}),pn({key:"C11",name:"Library card",skin:"lib",d:"The back is an old library card: the space is the shelf, every tag a ruled line with the day it was kept stamped beside it. When it goes, a big Kept stamp lands across it.",tpl:(n,t)=>`<div class="lh"><small>Shelf</small>${ls(n)}</div>${os(n,t)}<div class="rows">${Ri(n,{hash:!1,pre:()=>'<span class="dt">5 Nov</span>'})}<button type="button" class="add">+ Add a line</button></div>${Ws(n,2,"Remarks")}`,fx:()=>({add:n=>{let e=cs(n)?.querySelector(".dt");e&&It(e,[{transform:"rotate(-3deg) scale(1.8)",opacity:0},{transform:"rotate(-3deg) scale(1)",opacity:.8}],140),Z.play("stamp")},remove:n=>(Z.play("paper"),hs(n,[{opacity:1,height:"34px"},{opacity:0,height:"0px"}],200))}),leave:async(n,t,e)=>{let i=We('<div class="kept">Kept \xB7 5 Nov</div>');n.appendChild(i),Z.play("stamp"),Ys(.05),await It(i,[{opacity:0,transform:"translate(-50%,-50%) rotate(-12deg) scale(1.6)"},{opacity:.85,transform:"translate(-50%,-50%) rotate(-12deg) scale(1)"}],150,{ease:"cubic-bezier(.6,0,.9,.5)"}).finished,await ke(420,e)}}),pn({key:"C12",name:"Just the tags",skin:"min",note:!1,d:"The back with nothing but what you can change, big enough for a thumb: the space, the tags, a round plus to add one. No labels, no note.",tpl:(n,t)=>`<div class="row">${ls(n)}</div>${os(n,t)}<div class="row">${Ri(n)}<button type="button" class="add" aria-label="Add tag">+</button></div>`}));document.head.appendChild(We(`<style>
.tagp.pool { background: #0B2227; padding: 12px; color: #F4F2EF; }
.tagp.pool .ted { gap: 10px; }
.tagp.pool .sps { display: flex; gap: 5px; flex: none; }
.tagp.pool .sp { flex: 1 1 auto; height: 34px; padding: 0 6px; border: 0; border-radius: 11px; display: flex; align-items: center; justify-content: center; gap: 5px; font-size: 12.5px; white-space: nowrap; background: rgba(244,242,239,.05); color: #CFE0DE; cursor: pointer; }
.tagp.pool .sp[aria-pressed="true"] { background: #F4F2EF; color: #0E333A; font-weight: 600; }
.tagp.pool .sp i, .tagp.pool .rw .dot i { width: 8px; height: 8px; border-radius: 4px; flex: none; }
.tagp.pool .srch { display: flex; align-items: center; gap: 8px; height: 40px; padding: 0 5px 0 12px; border-radius: 12px; background: rgba(244,242,239,.08); flex: none; }
.tagp.pool .srch input { flex: 1; min-width: 0; border: 0; outline: 0; background: transparent; color: #F4F2EF; font-size: 16px; }
.tagp.pool .srch input::placeholder { color: #8FA9AC; }
.tagp.pool .srch .mk { height: 30px; padding: 0 12px; border: 0; border-radius: 9px; background: var(--rc); color: #0B2227; font-size: 13.5px; font-weight: 700; cursor: pointer; }
.tagp.pool .grid { flex: 1; min-height: 0; overflow-y: auto; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); grid-auto-rows: 78px; gap: 7px; align-content: start; }
.tagp.pool .tl { position: relative; border: 0; border-radius: 14px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; padding: 0 4px; color: #F4F2EF; font-size: 12.5px; background: rgba(244,242,239,.04); cursor: pointer; }
.tagp.pool .tl[aria-pressed="true"] { background: rgba(244,242,239,.13); box-shadow: inset 0 0 0 2px var(--rc); }
.tagp.pool .tl .ic { width: 34px; height: 34px; border-radius: 10px; display: grid; place-items: center; background: rgba(244,242,239,.08); }
.tagp.pool .tl[aria-pressed="true"] .ic { background: var(--rc); }
.tagp.pool .tl .ic svg { stroke: #CFE0DE; } .tagp.pool .tl[aria-pressed="true"] .ic svg { stroke: #0B2227; }
.tagp.pool .tl .nm { max-width: 100%; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.tagp.pool .ckb { position: absolute; top: 7px; right: 7px; width: 18px; height: 18px; border-radius: 5px; display: grid; place-items: center; box-shadow: inset 0 0 0 1.5px rgba(244,242,239,.3); }
.tagp.pool [aria-pressed="true"] > .ckb { background: var(--rc); box-shadow: none; }
.tagp.pool .ckb svg { opacity: 0; } .tagp.pool [aria-pressed="true"] > .ckb svg { opacity: 1; }
.tagp.pool .save { flex: none; position: relative; overflow: hidden; height: 46px; border: 0; border-radius: 23px; background: #F4F2EF; color: #0E333A; font-size: 15px; font-weight: 600; cursor: pointer; }
.tagp.pool .save .left, .tagp.pool .bar .left { position: absolute; top: 0; bottom: 0; left: 0; width: 100%; background: rgba(14,51,58,.16); }
.tagp.pool .save span, .tagp.pool .bar .dn span { position: relative; }
.tagp.pool .pop { animation: pkpop .32s cubic-bezier(.3,1.5,.5,1); }
@keyframes pkpop { from { transform: scale(.86); } to { transform: none; } }

.tagp.pool.sel { padding: 0; }
.tagp.pool.sel .ted { gap: 0; position: relative; }
.tagp.pool.sel .srch { margin: 12px 12px 4px; border-radius: 20px; }
.tagp.pool.sel .lst { flex: 1; min-height: 0; overflow-y: auto; padding-bottom: 76px; }
.tagp.pool.sel .hd { padding: 10px 16px 3px; font-size: 12px; color: #8FA9AC; }
.tagp.pool.sel .rw { width: 100%; height: 42px; display: flex; align-items: center; gap: 11px; padding: 0 16px; border: 0; background: transparent; color: #F4F2EF; font-size: 15px; text-align: left; cursor: pointer; }
.tagp.pool.sel .rw:active { background: rgba(244,242,239,.08); }
.tagp.pool.sel .rw .nm { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.tagp.pool.sel .rw .ckb { position: static; width: 20px; height: 20px; flex: none; }
.tagp.pool.sel .rw .rad { width: 20px; height: 20px; flex: none; border-radius: 10px; display: grid; place-items: center; box-shadow: inset 0 0 0 1.5px rgba(244,242,239,.38); }
.tagp.pool.sel .rw[aria-pressed="true"] .rad { box-shadow: inset 0 0 0 2px var(--rc); }
.tagp.pool.sel .rw .rad i { width: 9px; height: 9px; border-radius: 5px; }
.tagp.pool.sel .rw[aria-pressed="true"] .rad i { background: var(--rc); }
.tagp.pool.sel .mkr { color: var(--rc); font-weight: 600; }
.tagp.pool.sel .bar { position: absolute; left: 10px; right: 10px; bottom: 10px; height: 58px; border-radius: 17px; background: #F4F2EF; color: #0E333A; display: flex; align-items: center; gap: 10px; padding: 0 7px 0 14px; box-shadow: 0 10px 26px rgba(0,0,0,.45); }
.tagp.pool.sel .bar .ct { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.tagp.pool.sel .bar .ct b { font-size: 15px; font-weight: 700; }
.tagp.pool.sel .bar .ct small { font-size: 12.5px; color: #3E5A5E; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.tagp.pool.sel .bar .dn { position: relative; overflow: hidden; height: 44px; padding: 0 20px; border: 0; border-radius: 13px; background: #0E333A; color: #F4F2EF; font-size: 15px; font-weight: 600; cursor: pointer; }
.tagp.pool.sel .bar .dn .left { background: rgba(244,242,239,.2); }
.tagp.pool.sel .bump { animation: pkbump .26s cubic-bezier(.3,1.5,.5,1); }
@keyframes pkbump { from { transform: translateY(4px) scale(.97); } to { transform: none; } }
</style>`));var a_=["film camera","1970s","rangefinder","35mm","analog","vintage","secondhand","photography","gear","wishlist","gift idea"],o_={"film camera":"M4 8h3l2-2.5h6L17 8h3v11H4zM15.5 13a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0z","1970s":"M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7v5l3 2",rangefinder:"M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM12 3v3M12 18v3M3 12h3M18 12h3","35mm":"M5 3h14v18H5zM5 7h14M5 17h14M9 3v4M15 3v4M9 17v4M15 17v4",analog:"M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 12l4.5-4.5M7 12h1M12 7v1",vintage:"M7 3h10M7 21h10M8 3v2a4 4 0 0 0 8 0V3M8 21v-2a4 4 0 0 1 8 0v2",secondhand:"M20 11a8 8 0 0 0-14.5-4.5L4 8M4 4v4h4M4 13a8 8 0 0 0 14.5 4.5L20 16M20 20v-4h-4",photography:"M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM14.3 3.3L9 12.5M20.6 9.5H10M18 18.4l-5.3-9.2M9.7 20.7L15 11.5M3.4 14.5H14M6 5.6l5.3 9.2",gear:"M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9L7 7M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1",wishlist:"M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z","gift idea":"M4 10h16v10H4zM3 7h18v3H3zM12 7v13M12 7c-2-4-6-3-5 0M12 7c2-4 6-3 5 0"},l_="M3 12V4h8l10 10-8 8zM7.5 8.5h.01",$f='<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#0B2227" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5"/></svg>',c_='<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="#8FA9AC" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><circle cx="11" cy="11" r="6.5"/><path d="M16 16l4.5 4.5"/></svg>';function Yf(n,t,e){let i=We('<div class="ted"></div>'),s=[],r="",a="",o=()=>s.concat(a_.filter(S=>s.indexOf(S)<0)),l=S=>n.ts.tags.indexOf(S)>=0,c=e==="grid"?'<div class="sps"></div>':"";i.innerHTML=`${c}<label class="srch">${c_}<input placeholder="${e==="grid"?"Search tags":"Search or add tags"}" aria-label="Search or add tags" enterkeyhint="enter" autocomplete="off">${e==="grid"?'<button type="button" class="mk" hidden>Add</button>':""}</label>
    <div class="${e==="grid"?"grid":"lst"}"></div>
    ${e==="grid"?'<button type="button" class="save"><i class="left"></i><span></span></button>':'<div class="bar"><span class="ct"><b></b><small></small></span><button type="button" class="dn"><i class="left"></i><span>Done</span></button></div>'}`;let h=i.querySelector("input"),u=i.querySelector(".grid, .lst"),d=i.querySelector(".mk"),f=i.querySelector(".sps"),g=()=>n.ts.tags.length,x=S=>S+(S===1?" tag":" tags"),m=()=>!!r&&!o().some(S=>S.toLowerCase()===r.toLowerCase()),p=()=>o().filter(S=>!r||S.toLowerCase().indexOf(r.toLowerCase())>=0),T=()=>{e==="grid"?(f.innerHTML=dn.map(S=>`<button type="button" class="sp" data-s="${S.label}" aria-pressed="${S===n.ts.space}"><i style="background:${S.col}"></i>${S.label}</button>`).join(""),d.hidden=!m(),u.innerHTML=p().map(S=>`<button type="button" class="tl${S===a?" pop":""}" data-t="${S}" aria-pressed="${l(S)}"><span class="ic"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${o_[S]||l_}"/></svg></span><span class="nm">${S}</span><span class="ckb">${$f}</span></button>`).join(""),i.querySelector(".save span").textContent="Save "+x(g())):(u.innerHTML=`<div class="hd">Space</div>${dn.map(S=>`<button type="button" class="rw" data-s="${S.label}" aria-pressed="${S===n.ts.space}"><span class="rad"><i></i></span><span class="nm">${S.label}</span><span class="dot"><i style="display:block;background:${S.col}"></i></span></button>`).join("")}
        <div class="hd">Tags</div>${m()?`<button type="button" class="rw mkr" data-make="1"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg><span class="nm">Add \u201C${Cl(r)}\u201D</span></button>`:""}
        ${p().map(S=>`<button type="button" class="rw" data-t="${S}" aria-pressed="${l(S)}"><span class="ckb">${$f}</span><span class="nm">${S}</span></button>`).join("")}`,i.querySelector(".bar b").textContent=g()+(g()===1?" tag on it":" tags on it"),i.querySelector(".bar small").textContent="in "+n.ts.space.label)},M=()=>{let S=i.querySelector(".bar");S&&(S.classList.remove("bump"),S.offsetWidth,S.classList.add("bump"))},y=()=>{t.hold(),$r(n),Z.play("tick",1500),n.s.vrx-=.5,M()},D=()=>{let S=Cl(r.trim()).slice(0,24);S&&(o().indexOf(S)<0&&s.unshift(S),l(S)||n.ts.tags.push(S),r="",h.value="",a=S,T(),y(),Z.play("pop"))};return T(),i.addEventListener("pointerdown",()=>t.hold()),i.addEventListener("click",S=>{S.stopPropagation();let C=S.target.closest("button");if(C){if(C.classList.contains("save")||C.classList.contains("dn")){t.close();return}if(C.classList.contains("mk")||C.dataset.make){D();return}if(C.dataset.s){n.ts.space=dn.find(P=>P.label===C.dataset.s),T(),y();return}if(C.dataset.t){let P=C.dataset.t;n.ts.tags=l(P)?n.ts.tags.filter(w=>w!==P):n.ts.tags.concat([P]),a=P,T(),y()}}}),h.addEventListener("input",()=>{r=h.value.trim(),a="",t.hold(),T()}),h.addEventListener("keydown",S=>{S.stopPropagation(),S.key==="Enter"&&(S.preventDefault(),D())}),i._left=i.querySelector(".left"),i}Xa.push(pn({key:"M9",name:"Icon grid",skin:"pool",sc:1.6,y:380,pad:10,timer:!0,d:"Your pick from the card backs. The back is a grid of tag tiles with icons, the find\u2019s own already ticked; tap a tile to tick or untick it, search to find one or add your own, and the spaces sit along the top. Save drains with the time left.",editor:(n,t)=>Yf(n,t,"grid")}),pn({key:"M10",name:"Selection bar",skin:"pool sel",sc:1.6,y:380,pad:10,timer:!0,d:"Your other pick from the card backs. The back is a plain list: the space as one choice, then every tag with a tick box. A bar along the bottom counts what is on the find and bumps with each change; Done drains with the time left.",editor:(n,t)=>Yf(n,t,"bar")}));Ul.push({name:"Shooting star",from:"Genshin Impact wishes",look:"celestial",spin:"drop",d:"The page sinks into a night sky with drifting nebulae and dark hills. Your stars join into a constellation in the shape of the Inventory gem, one star breaks off and streaks across the sky, changing colour in flight. It crashes in a white-out that lights the hills, and the card spins out of the light as a real 3D card.",async play(n){oe.skyIn("#020A0F","#040E10");let t=rp("linear-gradient(180deg, #020A0F 0%, #06181D 40%, #0D2E34 75%, #1E4E55 100%)"),e=Vr('<div style="position:absolute;inset:-12%;background:radial-gradient(38% 26% at 28% 28%, rgba(63,122,130,.45), transparent 70%), radial-gradient(30% 22% at 74% 20%, rgba(110,80,130,.32), transparent 70%), radial-gradient(46% 26% at 60% 52%, rgba(30,78,85,.4), transparent 70%), radial-gradient(20% 12% at 40% 46%, rgba(167,211,204,.12), transparent 70%);opacity:0"></div>');It(e,[{opacity:0},{opacity:1}],900),It(e,[{transform:"translate(0,0) scale(1)"},{transform:"translate(-14px,8px) scale(1.06)"}],9e3,{ease:"ease-in-out"});let i=Vr(`<svg width="360" height="230" viewBox="0 0 360 230" style="position:absolute;left:0;bottom:0;opacity:0;overflow:visible">
        <path d="M0 110 C40 92 70 104 104 86 C140 66 172 96 212 80 C252 64 292 90 360 70 V330 H0Z" fill="#0A2226"/><path class="rim" d="M0 110 C40 92 70 104 104 86 C140 66 172 96 212 80 C252 64 292 90 360 70" fill="none" stroke-width="1.5" opacity="0"/>
        <path d="M0 150 C50 128 90 146 140 124 C190 104 230 136 280 120 C320 108 340 118 360 112 V330 H0Z" fill="#071A1D"/><path class="rim" d="M0 150 C50 128 90 146 140 124 C190 104 230 136 280 120 C320 108 340 118 360 112" fill="none" stroke-width="1.5" opacity="0"/>
        <path d="M0 192 C60 170 110 186 170 168 C230 150 280 178 360 160 V330 H0Z" fill="#040E10"/><path class="rim" d="M0 192 C60 170 110 186 170 168 C230 150 280 178 360 160" fill="none" stroke-width="2" opacity="0"/></svg>`);It(i,[{opacity:0,transform:"translateY(40px)"},{opacity:1,transform:"none"}],800);let s=vs({kind:"glow",x:180,y:800,r:300,color:"#2D6A72",a:0,sy:.5});De(s,{a:.6},600);let r=El(()=>ys(Gt(10,350),Gt(30,480),"#A7D3CC",1,1,{life:2.6,size:1.2}),18,60);for(let S=0;S<26;S++)ys(Gt(10,350),Gt(30,480),"#A7D3CC",1,1,{life:2.4,size:1.1,delay:.6});Pu(1.02,600,-10),await ke(420,n);let a=[[180,70],[222,104],[180,176],[138,104],[160,88],[200,88]],o=Vr(`<svg width="360" height="300" style="position:absolute;left:0;top:0;overflow:visible"><path d="M180 70 L222 104 L180 176 L138 104 Z M138 104 H222 M160 88 L180 104 L200 88" fill="none" stroke="#A7D3CC" stroke-opacity=".55" stroke-width="1" stroke-dasharray="420" stroke-dashoffset="420"/>${a.map(([S,C])=>`<circle cx="${S}" cy="${C}" r="2" fill="#F4F2EF"/>`).join("")}</svg>`);It(o.querySelector("path"),[{strokeDashoffset:420},{strokeDashoffset:0}],700,{ease:"ease-in-out"}),a.forEach(([S,C],P)=>ys(S,C,"#A7D3CC",1,1,{life:1.4,size:1.8,delay:P*.08})),Z.play("shimmer"),await ke(760,n),oe.keepRarity();let l=re[we],c=$v[we];i.querySelectorAll(".rim").forEach(S=>S.setAttribute("stroke",l));let h=vs({kind:"comet",x:180,y:70,color:c[0],w:3.4,trail:[],n:30});Dn(180,70,"#FFFFFF",16,160,{g:0}),Ui(180,70,"#FFFFFF",40,2,.4),Z.play("ting",0),Bu(o,600);let u={u:0},d=[180,70],f=[470,120],g=[180,362],x=S=>[(1-S)*(1-S)*d[0]+2*(1-S)*S*f[0]+S*S*g[0],(1-S)*(1-S)*d[1]+2*(1-S)*S*f[1]+S*S*g[1]],m=El(()=>{[h.x,h.y]=x(u.u),Dn(h.x,h.y,h.color,2,90,{g:60,life:.7,drag:1.5,white:.5}),Math.random()<.3&&ys(h.x,h.y,h.color,1,6,{life:.6})},120,10),p=1400+250*(c.length-1);Z.play("meteor",p/1e3+.2),Z.play("riser",p/1e3);let T=De(u,{u:1},p,S=>.55*S+.45*S*S),M=c.length===3?[.38,.68]:c.length===2?[.55]:[];for(let S=0;S<M.length;S++){for(;u.u<M[S];)await ke(16,n);let C=c[S+1];h.color=C,h.w+=1;let[P,w]=x(u.u);Z.play("ting",S+1),Ys(.12+.08*S),vs({kind:"flare",x:P,y:w,color:C,len:170+60*S,th:6,core:70,a:1,life:.55}),Ui(P,w,C,70+30*S,4,.5),Dn(P,w,C,24,260,{g:100}),vs({kind:"bloom",x:P,y:w,color:C,a:.4,R:260,life:.5})}for(;u.u<.9;)await ke(16,n);Z.play("suck",.25),await T,Ni(n),m.dead=!0,h.dead=!0,op(we),py();let y=Oy(180,362,{sc:.55});Ga(.6,.45),Ui(180,362,"#FFFFFF",260,5,.5);let D=Vy(y,{y:362,z:.3,onFace:()=>{Ga(.3,.35),i.querySelectorAll(".rim").forEach((S,C)=>It(S,[{opacity:0},{opacity:1,offset:.15},{opacity:.35}],1600,{delay:C*60}))}});Pu(1,900,0,"cubic-bezier(.2,.8,.2,1)"),await D,Ni(n),await Cy(n,y,{delay:400,pointsY:526}),r.dead=!0}});var oe=(()=>{let n=window.InvApp,t=(h,u)=>{try{n?n.post(h,u==null?"":JSON.stringify(u)):(window.__appLog=window.__appLog||[]).push([h,u])}catch{}},e=!1,i=!1,s=!1,r=null,a=!1,o=!1,l=document.getElementById("bands"),c=()=>{let h=Math.min(innerWidth/kn,innerHeight/Ln);Sn.style.transform=`translate(${((innerWidth-kn*h)/2).toFixed(2)}px, ${((innerHeight-Ln*h)/2).toFixed(2)}px) scale(${h})`};return c(),addEventListener("resize",c),{post:t,get skipped(){return i},get inTags(){return a},set inTags(h){a=h},get aborted(){return s},set aborted(h){s=h},set skipped(h){i=h},started:()=>o,begin(){o=!0},waitTags:()=>new Promise(h=>{r=h}),tagsDone(h){let u=r;r=null,u&&u(h||{})},skyIn(h,u,d=420){l.style.background=`linear-gradient(180deg, ${h} 0 50%, ${u} 50% 100%)`,It(l,[{opacity:0},{opacity:1}],d),t("sky",{a:1,ms:d})},skyOut(h){getComputedStyle(l).opacity!=="0"&&It(l,[{opacity:getComputedStyle(l).opacity},{opacity:0}],h),t("sky",{a:0,ms:h})},keepRarity(){e=!0,t("rarity",{r:we})},setRarity(h){!e&&re[h]&&(we=h)},cardRect(h){h.root.updateMatrixWorld(!0);let u=new I;h.root.getWorldPosition(u);let d=h.root.scale.x,f=qr.getBoundingClientRect(),g=(M,y)=>{let D=new I(u.x+M*d,u.y+y*d,u.z).project(qa);return[f.left+(D.x+1)/2*f.width,f.top+(1-D.y)/2*f.height]},[x,m]=g(-Ii/2,ds/2),[p,T]=g(Ii/2,-ds/2);return{x:(x+p)/2,y:(m+T)/2,w:p-x,h:T-m}}}})();tp=!1;Z.play=(n,t)=>{Dl||oe.post("sfx",{n,a:t??null,r:we})};Xa.push({key:"APP",name:"The app\u2019s tag step",d:"The card turns over and the app shows its icon grid on the back.",async run(n,t){if(oe.skipped)return;let e=t.s,i=t.flipT,s={rx:e.rx,ry:e.ry,rz:e.rz,fl:e.fl,y:t.y,sc:e.sc};t.interactive=!1,t.idle=!1,t.ptr=null,Pl(t,!0);let r=oe.cardRect(t);Z.play("whoosh"),await Ce(230,o=>{e.fl=wt(s.fl,i+Math.PI/2,o),e.rx=wt(s.rx,0,o),e.ry=wt(s.ry,0,o),e.rz=wt(s.rz,0,o)},ee.in),Ni(n),t.vis=0,oe.inTags=!0,oe.post("tags",{rect:r,turnMs:230});let a=await oe.waitTags();oe.inTags=!1,Ni(n),(a.space!==void 0||a.tags)&&(t.ts.space=a.space?{label:a.space.label,col:a.space.col}:null,t.ts.tags=(a.tags||[]).slice(0,12),$r(t)),e.fl=i+Math.PI*1.5,t.vis=1,await Ce(230,o=>{e.fl=wt(i+Math.PI*1.5,i+Math.PI*2,o)},ee.out),Ni(n),Pl(t,!1),t.flipT=e.fl,t.k=70,t.d=9,t.idle=!0,t.interactive=!0}});gp=Xa.length-1;rp=n=>{let t=Vr(`<div style="position:absolute;inset:-64px;background:${n};opacity:0"></div>`);return It(t,[{opacity:0},{opacity:1}],420),t};var Oa=[],Zf=document.getElementById("flash");Ga=(n=.9,t=.25,e="#FFFFFF")=>{Oa.push({alpha:n,dur:t,color:e,t:0})};Ll(n=>{let t=n/1e3/Bi*Wr(),e=0,i="#FFFFFF";for(let s=Oa.length-1;s>=0;s--){let r=Oa[s];r.t+=t;let a=r.t/r.dur;if(a>=1){Oa.splice(s,1);continue}let o=r.alpha*(1-a)*(1-a);o>e&&(e=o,i=r.color)}Zf.style.opacity=e.toFixed(3),e&&(Zf.style.background=i)});function Fl(){Tn++,Dl=!1,Ks=!1,Uu=1,za=1,bl=null,xs=0,Xs=[],hp(),Ml.innerHTML="",Nu.innerHTML="",kl.innerHTML="",$s=[],Ha=[],Sl=[],wl=[],Va=[],Li.length=0,Oa.length=0,[wn,ka,qr].forEach(n=>n.getAnimations().forEach(t=>t.cancel())),wn.style.transform="",wn.style.translate="",wn.style.rotate=""}var h_=$a;$a=(n=160)=>{oe.skyOut(n),h_(n)};var u_=Nl;Nl=async()=>{oe.skipped||oe.aborted||oe.inTags||(oe.skipped=!0,oe.post("skip"),await u_(),oe.aborted||oe.post("done",{skipped:!0}))};document.addEventListener("click",()=>{Ks&&!oe.inTags&&Nl()});async function d_(){Fl();let n=Tn;Ks=!0;try{await ke(60,n),Dy(),await Ul[0].play(n)}catch(t){t instanceof Ms||(console.error(t),oe.post("fail",{m:String(t&&t.message||t)}))}n===Tn&&(Fl(),!oe.skipped&&!oe.aborted&&oe.post("done",{skipped:!1}))}function Jf(n){n&&(n.title!=null&&(Di.title=n.title),n.kind!=null&&(Di.kind=n.kind),n.glyph!=null&&(Di.glyph=n.glyph),n.no!=null&&(Di.no=n.no),n.space!==void 0&&(dn.length=0,n.space&&dn.push({label:n.space.label,col:n.space.col,n:0})),n.tags&&(Fu.length=0,Fu.push(...n.tags.slice(0,12))),n.count!=null&&(Gr.before=n.count),n.saved!=null&&(Gr.saved=n.saved))}var Gr={before:null,saved:1},Sp=n=>Gr.before==null?"":(Gr.before+(n?Gr.saved:0)).toLocaleString("en-GB"),Tu=!1;window.InvMoment={start(n={}){if(!oe.started()){if(oe.begin(),Jf(n.find),we=1,n.rarity&&oe.setRarity(n.rarity),!Tu){let t=Xv();un.src=t,new wa().load(t,e=>{Tu||(_s.image=e.image,_s.needsUpdate=!0)})}d_()}},rarity(n){oe.setRarity(n)},find(n){Jf(n)},photo(n){let t=new Image;t.onload=()=>{Tu=!0,un.src=n,_s.image=t,_s.needsUpdate=!0,ws.forEach(e=>e.redraw&&e.redraw())},t.src=n},tagsDone(n){oe.tagsDone(n)},skip(){Ks&&!oe.inTags&&Nl()},abort(){if(oe.aborted)return;oe.aborted=!0;let n=++Tn;Ks=!1,oe.tagsDone(null),$a(300),Zs(()=>{n===Tn&&Fl(),oe.post("done",{aborted:!0})},340)}};Promise.all([document.fonts.load('600 16px "Google Sans Flex"'),document.fonts.load('500 11px "Google Sans Flex"'),window.__warm||Promise.resolve()]).then(()=>oe.post("ready",{w:innerWidth,h:innerHeight,dpr:devicePixelRatio}),()=>oe.post("ready",{}));})();
