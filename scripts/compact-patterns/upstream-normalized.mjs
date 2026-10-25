// SPDX-License-Identifier: MPL-2.0
import {pathToFileURL} from 'node:url';
const Bend = await import(pathToFileURL(process.argv[2]).href);
const book=Bend.book_nil(); await Bend.book_load(book,process.argv[3],'',new Map());
Bend.book_valid(book);
for (const name of ['is_hi','main','open']) {
 const def=book.tlds[name]; if (!def?.v) continue;
 const body=Bend.term_lower(Bend.term_snf(book,def.v));const stack=[[body,0]];let max=0,nodes=0;
 while(stack.length){const [term,depth]=stack.pop();max=Math.max(max,depth);nodes++;
 const keys={All:['A','B'],Lam:['f'],App:['f','x'],Mat:['h','m'],Typ:['g'],Min:['a','b'],Eql:['a','b','T'],Rwt:['e','p','f'],Ann:['x','T'],Sub:['v','f'],Let:['v','f'],Ctr:['x'],ADT:['x']}[term.$]??[];
 for(const k of keys)for(const child of Array.isArray(term[k])?term[k]:[term[k]]){if(child?.$)stack.push([child,depth+1]);}
 }
 console.log(JSON.stringify({name,max_depth:max,nodes,body,canonical:canonical(body,[])}));
}

function canonical(t,b) {
 switch(t.$){
 case 'Var': {const i=b.toReversed().indexOf(t.i); if(i<0)throw Error('unbound');return `Var:${i};`;}
 case 'Ref': return `Ref:${t.k};`;
 case 'Lam': return 'Lam;'+canonical(t.f,[...b,t.i]);
 case 'App': return 'App;'+canonical(t.f,b)+canonical(t.x,b);
 case 'Mat': return `Mat:${t.k};`+canonical(t.h,b)+canonical(t.m,b);
 case 'Ctr': return `Ctr:${t.k}:${t.x.length};`+t.x.map(x=>canonical(x,b)).join('');
 case 'Efq': return 'Efq;';
 default: throw Error('unsupported '+t.$);
 }
}
