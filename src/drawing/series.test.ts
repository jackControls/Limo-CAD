import type {DrawingAnnotationDto, DrawingProjectionDto, DrawingSheetDto, DrawingViewDto} from '../engine/types';
import {buildDrawingSheetDxf} from './dxf';
import {buildDrawingSheetSvg} from './export';
import {defaultDrawingSheetStyle} from './sheet';

function check(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
const view: DrawingViewDto = {id:1,name:'Top',kind:'top',direction:[0,0,1],up:[0,1,0],position:[75,40],scale:1,
  scope:'definition',occurrence_ids:[],body_ids:[],show_hidden_lines:false,show_tangent_edges:false,
  parent_view_id:null,alignment:'free',derivation:null};
const points: [number,number][] = [[0,0],[20,0],[50,0]];
const projection: DrawingProjectionDto = {visible:[],hidden:[],circles:[],section:[],bounds:[0,0,50,20],
  anchors:points.map((point,index)=>({body_id:1,edge_id:index+1,edge_key:`e${index+1}`,endpoint:'start',
    occurrence_id:null,model_point:[point[0],point[1],0],point,hidden:false}))};
const anchors = projection.anchors.map(a=>({body_id:a.body_id,edge_id:a.edge_id,edge_key:a.edge_key,endpoint:a.endpoint,
  occurrence_id:null,circle_center:false,topology_signature:null,fallback_point:a.model_point}));
const sheet: DrawingSheetDto = {id:1,name:'Series parity',format:'a4',orientation:'landscape',views:[view],annotations:[],
  standard:'iso',projection_method:'third_angle',tolerance_note:{preset:'iso2768_medium',custom:''},
  title_block:{title:'Series parity',drawing_number:'',revision:'',author:'',checked_by:'',approved_by:'',company:'',material:'',finish:''},
  style:defaultDrawingSheetStyle(),template_name:'',revisions:[],bom:[],release:{status:'draft',released_revision:'',released_at:''},
  revision_table_position:null,bom_table_position:null};
const presentation = {tolerance:{mode:'none' as const,upper:0,lower:0},basic:false,reference:false,fit_class:'',dual_units:null};

for (const layout of ['chain','baseline','continued'] as const) {
  for (const spacing of [7,11]) {
    const annotation: DrawingAnnotationDto = {kind:'chain_dimension',id:1,view_id:1,anchors,mode:'horizontal',layout,
      offset:12,spacing,prefix:'',suffix:'',precision:2,presentation};
    const input={...sheet,annotations:[annotation]}, before=JSON.stringify(input);
    const svg=buildDrawingSheetSvg(input,[projection]);
    const groups=[...svg.matchAll(/<g data-dimension-arrows="[\s\S]*?<\/g>/g)].map(m=>m[0]);
    check(groups.length===2,`${layout}: both SVG series spans survive`);
    const expected=[62,layout==='baseline'?62+spacing:62];
    groups.forEach((group,index)=>{
      const tips=[...group.matchAll(/<polygon points="([^\"]+)"/g)].map(m=>m[1].split(/\s+/)[0].split(',').map(Number));
      check(tips.length===2&&tips.every(tip=>Math.abs(tip[1]-expected[index])<1e-6),
        `${layout}: SVG span ${index} must match release React offset ${expected[index]}, got ${JSON.stringify(tips)}`);
    });
    const tokens=buildDrawingSheetDxf(input,[projection]).trim().split(/\r?\n/);
    const entities:Array<Map<string,string>>=[];
    let entity=new Map<string,string>();
    for(let i=0;i<tokens.length;i+=2){const key=tokens[i].trim(),value=tokens[i+1];if(key==='0'){if(entity.size)entities.push(entity);entity=new Map();}entity.set(key,value);}
    entities.push(entity);
    const lines=entities.filter(e=>e.get('0')==='LINE'&&e.get('8')==='DIMENSIONS');
    check(lines.length===2,`${layout}: DXF retains both series lines`);
    lines.forEach((line,index)=>check(Math.abs(Number(line.get('20'))-(210-expected[index]))<1e-6
      &&Math.abs(Number(line.get('21'))-(210-expected[index]))<1e-6,`${layout}: DXF span ${index} changed release React offset`));
    check(JSON.stringify(input)===before,`${layout}: exporting changed saved series intent`);
  }
}
console.log('Chain, Baseline, and Continued SVG/DXF offsets match the browser drawing workspace.');
